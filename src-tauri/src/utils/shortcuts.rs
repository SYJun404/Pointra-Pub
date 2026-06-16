use crate::{
    utils::{
        define_config::{keycode_from_str, AppConfig},
        get_text::{get_data_from_selected_text, get_word_under_cursor},
        show_window::show_input_window,
    },
    AppState,
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// 初始化Point快捷键
pub fn init_point_listener(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let device_state = DeviceState::new();
        let mut last_state = false; // false 代表松开，true 代表按下
        let mut cached_window = None;

        let current_key_str = app_handle
            .state::<AppState>()
            .config
            .lock()
            .unwrap()
            .point_key
            .clone();
        let current_key = keycode_from_str(&current_key_str).unwrap_or(Keycode::F2);

        loop {
            let keys = device_state.get_keys();

            let is_pressed = keys.contains(&current_key);

            // 只有当状态发生改变时才触发逻辑
            if is_pressed != last_state {
                last_state = is_pressed;

                if is_pressed {
                    // 按下：显示并聚焦
                    if cached_window.is_none() {
                        cached_window = app_handle.get_webview_window("main");
                    }
                    if let Some(win) = &cached_window {
                        get_word_under_cursor(app_handle.state(), win.clone());
                    }
                } else {
                    let app_state = app_handle.state::<AppState>();
                    let is_hovered = app_state.window_locked.load(Ordering::Relaxed);
                    if !is_hovered {
                        if cached_window.is_none() {
                            cached_window = app_handle.get_webview_window("main");
                        }
                        if let Some(win) = &cached_window {
                            let _ = win.hide();
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(60));
        }
    });
}

/// 初始化快捷键
pub fn init_shortcuts<R: Runtime>(app: &AppHandle<R>) {
    let shortcut_manager = app.global_shortcut();
    let app_state = app.state::<AppState>();
    let config = app_state.config.lock().unwrap();

    let shortcut_select_text =
        Shortcut::new(Some(config.select_text_modifiers), config.select_text_code);
    let shortcut_show_input =
        Shortcut::new(Some(config.search_win_modifiers), config.search_win_code);

    let _ = shortcut_manager.register(shortcut_select_text);
    let _ = shortcut_manager.register(shortcut_show_input);
}

/// 处理快捷键事件
pub fn handle_shortcut_event(app: &AppHandle, shortcut: &Shortcut) {
    let app_state = app.state::<AppState>();
    let config = app_state.config.lock().unwrap();

    if let Some(win) = app.get_webview_window("main") {
        if shortcut.matches(config.select_text_modifiers, config.select_text_code) {
            get_data_from_selected_text(win, app);
        } else if shortcut.matches(config.search_win_modifiers, config.search_win_code) {
            show_input_window(win);
        }
    }
}

/// 重启两类快捷键
pub fn restart_shortcuts<R: Runtime>(app: &AppHandle<R>, new_config: AppConfig) {
    {
        let app_state = app.state::<AppState>();
        let mut config = app_state.config.lock().unwrap();
        *config = new_config;
    }

    init_shortcuts(app);
}
