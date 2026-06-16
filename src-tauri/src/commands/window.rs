use crate::utils::update::{check_for_update, UpdateCheckResponse};
use crate::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

// 更新鼠标是否在窗口内
#[tauri::command]
pub fn update_hover_status(hovered: bool, state: State<'_, AppState>) {
    state.window_locked.store(hovered, Ordering::Relaxed);
}

// 停止快捷键
#[tauri::command]
pub fn stop_shortcuts(app: AppHandle) {
    let _ = app.global_shortcut().unregister_all();
}

// 检查更新
#[tauri::command]
pub async fn trigger_update_check(version: String) -> Result<UpdateCheckResponse, ()> {
    if let Some((update_info, has_update)) = check_for_update(&version).await {
        if has_update {
            return Ok(update_info);
        }
    }
    Err(())
}
