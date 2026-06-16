use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::{thread, time::Duration};
use tauri::WebviewWindow;

pub fn get_text(window: &WebviewWindow) -> Option<(String, i32, i32)> {
    let mut enigo = Enigo::new(&Settings::default()).ok()?;
    let mut clipboard = Clipboard::new().ok()?;

    // 保存当前剪贴板内容，用于后续恢复
    let old_content = clipboard.get_text().ok();

    // 清空剪贴板，以便判断 Cmd+C 是否触发了新内容
    let _ = clipboard.set_text("");

    // 模拟 Command + C
    let _ = window.run_on_main_thread(move || {
        let _ = enigo.key(Key::Meta, Press);
        let _ = enigo.key(Key::Unicode('c'), Click);
        // let _ = enigo.key(Key::Unicode('c'), Release);
        let _ = enigo.key(Key::Meta, Release);
    });

    thread::sleep(Duration::from_millis(45));

    let text_opt = clipboard
        .get_text()
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| {
            if let Some(old) = old_content {
                let _ = clipboard.set_text(old);
            }
            None
        });
    text_opt.map(|text| (text, 16, 18))
}
