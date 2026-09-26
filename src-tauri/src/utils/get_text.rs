#[cfg(target_os = "macos")]
use super::selection_mac::get_text;
#[cfg(target_os = "macos")]
use crate::utils::capture_mac::capture_around_cursor;
#[cfg(target_os = "macos")]
use crate::utils::ocr_mac::{recognize_words, select_word};

#[cfg(target_os = "windows")]
use super::selection_win::get_text;
#[cfg(target_os = "windows")]
use crate::utils::capture_win::capture_around_cursor;
#[cfg(target_os = "windows")]
use crate::utils::ocr_win::{recognize_words, select_word};

use crate::utils::show_window::show_main_window;
use crate::AppState;

use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

/// 取词来源标记：所有取词路径都失败、最后由「鼠标所在位置 OCR」兜底
pub const SOURCE_OCR: &str = "ocr";

/// 防止连按 Alt+1 时多个取词任务并发竞争共享状态（剪贴板序列号基准等）
static PICK_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// RAII 守卫：取词任务结束时释放防重入标志（即使 panic 也能释放）
struct PickGuard;
impl Drop for PickGuard {
    fn drop(&mut self) {
        PICK_IN_PROGRESS.store(false, Ordering::Release);
    }
}

pub fn get_data_from_selected_text(window: WebviewWindow, app: &AppHandle) {
    // 已有取词在进行中，忽略本次按键（避免并发竞态）
    if PICK_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let win_clone = window.clone();
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        let _guard = PickGuard; // 无论下面走哪条分支，闭包结束都会释放

        #[cfg(target_os = "macos")]
        let text_opt = get_text(&window).filter(|(text, _, _, _)| !text.trim().is_empty());
        #[cfg(target_os = "windows")]
        let text_opt = get_text().filter(|(text, _, _, _)| !text.trim().is_empty());

        if let Some((text, custom_x, custom_y, source)) = text_opt {
            #[cfg(debug_assertions)]
            eprintln!("[Pointra][取词] 采纳 source={source} text={text:?}");
            window
                .emit("from-cursor", json!({ "text": text, "source": source }))
                .ok();
            show_main_window(&window, custom_x, custom_y);
        } else {
            let app_state = app_clone.state::<AppState>();
            get_word_at_cursor_fallback(app_state, win_clone);
        }
    });
}

/// 鼠标位置 OCR 取一个词（按平台实现，未支持平台返回 None）
#[cfg(target_os = "macos")]
fn ocr_word_at_cursor(state: &AppState) -> Option<String> {
    let img = capture_around_cursor(200, 40).ok()?;
    let words = recognize_words(&img, &state.ocr_state).ok()?;
    select_word(&words).filter(|word| !word.is_empty())
}

#[cfg(target_os = "windows")]
fn ocr_word_at_cursor(state: &AppState) -> Option<String> {
    let img = capture_around_cursor(200, 40).ok()?;
    let words = recognize_words(&img, &state.ocr_state).ok()?;
    select_word(&words).filter(|word| !word.is_empty())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn ocr_word_at_cursor(_state: &AppState) -> Option<String> {
    None
}

pub fn get_word_under_cursor(app_state: State<'_, AppState>, window: WebviewWindow) {
    if let Some(word) = ocr_word_at_cursor(app_state.inner()) {
        #[cfg(debug_assertions)]
        eprintln!("[Pointra][取词] F2 指针 OCR 命中: {word:?}");
        show_main_window(&window, 0, 0);
        window
            .emit("from-cursor", json!({ "text": word, "source": SOURCE_OCR }))
            .ok();
    }
}

/// 选区 / 剪贴板 / UIA 鼠标取词全部失败后的最后兜底。
/// 注意：这里刻意使用 select_word 而不是 select_sentence ——
/// 在终端 / TUI 里「整句」会把鼠标所在整行甚至跨行的 OCR 噪声一起抓取，
/// 结果既不是用户选中的内容，又会把乱码送去翻译。
pub fn get_word_at_cursor_fallback(app_state: State<'_, AppState>, window: WebviewWindow) {
    if let Some(word) = ocr_word_at_cursor(app_state.inner()) {
        #[cfg(debug_assertions)]
        eprintln!("[Pointra][取词] 兜底 OCR 命中: {word:?}");
        #[cfg(target_os = "macos")]
        let (custom_x, custom_y) = (16, 18);
        #[cfg(not(target_os = "macos"))]
        let (custom_x, custom_y) = (23, 15);

        show_main_window(&window, custom_x, custom_y);
        window
            .emit("from-cursor", json!({ "text": word, "source": SOURCE_OCR }))
            .ok();
    }
}
