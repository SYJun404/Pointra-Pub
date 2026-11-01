#[cfg(target_os = "macos")]
use super::selection_mac::get_text;
#[cfg(target_os = "macos")]
use crate::utils::capture_mac::capture_at_position;
#[cfg(target_os = "macos")]
use crate::utils::ocr_mac::{recognize_words, select_word};

#[cfg(target_os = "windows")]
use super::selection_win::get_text;
#[cfg(target_os = "windows")]
use crate::utils::capture_win::capture_around_cursor;
#[cfg(target_os = "windows")]
use crate::utils::ocr_win::{recognize_words, select_word};

use crate::utils::show_window::show_translation_window;
use crate::AppState;
use std::time::Instant;
use tauri::WebviewWindow;

pub fn get_data_from_selected_text(window: WebviewWindow) {
    tauri::async_runtime::spawn(async move {
        #[cfg(target_os = "macos")]
        let text_opt = get_text(&window).filter(|(text, _, _)| !text.trim().is_empty());
        #[cfg(target_os = "windows")]
        let text_opt = get_text().filter(|(text, _, _)| !text.trim().is_empty());

        if let Some((text, custom_x, custom_y)) = text_opt {
            show_translation_window(&window, text, custom_x, custom_y);
        }
    });
}

/// Only called by the cursor worker; never block key polling on capture or Vision.
/// Keep the trigger position even when the mouse moves during a cold inference.
pub fn get_word_under_cursor(
    app_state: &AppState,
    position: (i32, i32),
    on_captured: impl FnOnce(),
) -> anyhow::Result<Option<String>> {
    let started = Instant::now();
    #[cfg(target_os = "macos")]
    let img = capture_at_position(position, 200, 40)?;
    #[cfg(target_os = "windows")]
    let img = {
        let _ = position;
        capture_around_cursor(200, 40)?
    };
    let captured = started.elapsed();
    // Show feedback only after capture, so the popup cannot cover the source text.
    on_captured();
    let words = recognize_words(&img, &app_state.ocr_state)?;
    if std::env::var_os("POINTRA_TRACE_CURSOR").is_some() {
        eprintln!(
            "[Cursor] capture={captured:?} ocr={:?}",
            started.elapsed() - captured
        );
    }
    Ok(select_word(&words).filter(|word| !word.is_empty()))
}
