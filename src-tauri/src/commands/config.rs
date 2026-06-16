use crate::utils::define_config::{get_config_from_store, save_config_to_store, AppConfig};
use crate::utils::shortcuts::restart_shortcuts;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    get_config_from_store(&app)
}

#[tauri::command]
pub async fn update_config(app: AppHandle, new_config: AppConfig) -> Result<bool, String> {
    save_config_to_store(&app, &new_config)?;
    restart_shortcuts(&app, new_config.clone());

    // macOS: 根据 auto_start 启用或禁用开机自启
    #[cfg(target_os = "macos")]
    {
        let autostart_manager = app.autolaunch();
        if new_config.auto_start {
            if !autostart_manager.is_enabled().unwrap_or(false) {
                autostart_manager
                    .enable()
                    .map_err(|e| format!("启用开机自启失败: {}", e))?;
            }
        } else {
            if autostart_manager.is_enabled().unwrap_or(false) {
                autostart_manager
                    .disable()
                    .map_err(|e| format!("禁用开机自启失败: {}", e))?;
            }
        }
    }

    // 传播new_config
    app.emit("win-event", json!({"type": "config", "data": new_config}))
        .ok();
    Ok(true)
}

#[tauri::command]
pub async fn complete_setup(app: AppHandle) -> Result<bool, String> {
    let mut config = get_config_from_store(&app).unwrap_or_default();
    config.first_setup_completed = true;
    save_config_to_store(&app, &config)?;

    // 重启应用（此调用不会返回)
    restart_app()
}

/// 重启当前应用
fn restart_app() -> ! {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).spawn();
    }
    std::process::exit(0);
}
