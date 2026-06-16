#![allow(unexpected_cfgs)]

mod commands;
mod utils;
use commands::access_mac::{check_permission, request_permission};
use commands::audio::{play_phonetic_url, AudioState};
use commands::config::{complete_setup, get_config, update_config};
use commands::translate::fetch_trans_res;
use commands::window::{stop_shortcuts, trigger_update_check, update_hover_status};
use reqwest::Client;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::ShortcutState;
use utils::define_config::{get_config_from_store, AppConfig};

#[cfg(target_os = "macos")]
use utils::{ocr_mac::OcrState, set_window::apply_window_effects};

#[cfg(target_os = "windows")]
use utils::ocr_win::OcrState;

use utils::shortcuts::{handle_shortcut_event, init_point_listener, init_shortcuts};
use utils::tray;

pub struct AppState {
    ocr_state: Arc<OcrState>,
    client: Client,
    window_locked: Arc<AtomicBool>,
    audio_state: AudioState,
    config: Mutex<AppConfig>,
}

pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::Builder::new().build());

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        // 注册命令
        .invoke_handler(tauri::generate_handler![
            update_hover_status,
            fetch_trans_res,
            play_phonetic_url,
            get_config,
            update_config,
            complete_setup,
            stop_shortcuts,
            check_permission,
            request_permission,
            trigger_update_check
        ])
        // 初始化应用状态
        .setup(|app| {
            let config = get_config_from_store(app.handle()).unwrap_or_default();

            // 先初始化 AppState（后续命令依赖它）
            app.manage(AppState {
                ocr_state: OcrState::new(),
                client: Client::new(),
                window_locked: Arc::new(AtomicBool::new(false)),
                audio_state: AudioState::new(),
                config: Mutex::new(config.clone()),
            });

            #[cfg(target_os = "macos")]
            {
                // macOS 首次启动检测权限
                if !config.first_setup_completed {
                    let (acc, screen) = unsafe {
                        (
                            crate::utils::macos_api::check_accessibility(),
                            crate::utils::macos_api::check_screen_recording(),
                        )
                    };

                    if !(acc && screen) {
                        // 权限未授予，打开设置界面引导用户配置
                        tray::create_setting_win(app.handle());
                        // 跳过托盘、快捷键等初始化，重启后会正常加载
                        return Ok(());
                    } else {
                        // 权限已全部授予，标记首次设置完成
                        let mut updated_config = config.clone();
                        updated_config.first_setup_completed = true;
                        let _ = crate::utils::define_config::save_config_to_store(
                            app.handle(),
                            &updated_config,
                        );
                    }
                }

                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                apply_window_effects(app.handle().clone());
            }

            // 确保自动启动状态与配置一致
            let autostart_manager = app.autolaunch();
            if config.auto_start && !autostart_manager.is_enabled().unwrap_or(false) {
                let _ = autostart_manager.enable();
            } else if !config.auto_start && autostart_manager.is_enabled().unwrap_or(false) {
                let _ = autostart_manager.disable();
            }

            // 初始化托盘
            tray::init(app)?;

            init_point_listener(app.handle().clone());
            init_shortcuts(app.handle());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        handle_shortcut_event(app, shortcut);
                    }
                })
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
