// 自定义一个宏来简化
macro_rules! cfg_mods {
    ($cfg:meta, $($mod:ident),+ $(,)?) => {
        $(#[cfg($cfg)] pub mod $mod;)+
    };
}

// 使用
cfg_mods!(
    target_os = "macos",
    capture_mac,
    macos_api,
    ocr_mac,
    set_window,
    selection_mac
);
cfg_mods!(target_os = "windows", capture_win, ocr_win, selection_win);

// 通用模块
pub mod define_config;
pub mod get_text;
pub mod ocr_common;
pub mod shortcuts;
pub mod show_window;
pub mod trans_server;
pub mod tray;
pub mod update;
