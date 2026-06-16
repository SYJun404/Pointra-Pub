#[cfg(target_os = "macos")]
use super::set_window::apply_squircle_corners;
use serde_json::json;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

pub fn create_setting_win<R: Runtime>(app: &AppHandle<R>) {
    let window_builder = WebviewWindowBuilder::new(
        app,
        "setting",
        WebviewUrl::App("src/setting/index.html".into()),
    )
    .title("Pointra")
    .inner_size(400.0, 600.0)
    .resizable(false)
    .visible(false)
    .decorations(false)
    .center()
    .transparent(true)
    .shadow(true);

    match window_builder.build() {
        Ok(window) => {
            #[cfg(target_os = "macos")]
            {
                apply_squircle_corners(&window, 16.0);
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
        Err(e) => {
            println!("Failed to build main window: {}", e);
        }
    }
}

/// 初始化系统托盘
pub fn init<R: Runtime>(app: &tauri::App<R>) -> Result<(), tauri::Error> {
    let handle = app.handle();

    // 1. 创建托盘菜单项
    let setting_item = MenuItem::with_id(handle, "setting", "设置界面", true, None::<&str>)?;
    let search_item = MenuItem::with_id(handle, "search", "翻译文本", true, None::<&str>)?;
    let restart_item = MenuItem::with_id(handle, "restart", "重新启动", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(handle, "quit", "退出软件", true, None::<&str>)?;

    // 将菜单项组合成菜单
    let menu = Menu::with_items(
        handle,
        &[&setting_item, &search_item, &restart_item, &quit_item],
    )?;

    // 2. 构建托盘
    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        // 响应菜单事件
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "search" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.emit("win-event", json!({"type": "search"})).ok();
                    let _ = window.center();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "restart" => {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).spawn();
                }
                std::process::exit(0);
            }
            "setting" => {
                if let Some(window) = app.get_webview_window("setting") {
                    let _ = window.close();
                } else {
                    create_setting_win(app);
                }
            }

            _ => {}
        })
        .build(app)?;

    Ok(())
}
