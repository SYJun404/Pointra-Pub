#[cfg(target_os = "macos")]
use crate::utils::macos_api;

#[tauri::command]
#[cfg(target_os = "macos")]
pub fn check_permission() -> (bool, bool) {
    #[cfg(target_os = "macos")]
    {
        (unsafe { macos_api::check_accessibility() }, unsafe {
            macos_api::check_screen_recording()
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        (true, true)
    }
}

#[tauri::command]
#[cfg(target_os = "macos")]
pub fn request_permission(kind: u32) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let url = match kind {
            1 => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            2 => "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
            _ => "",
        };
        let _ = Command::new("open").arg(url).spawn();
    }
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn check_permission() -> (bool, bool) {
    (true, true)
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn request_permission() {
    ()
}
