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

use crate::utils::show_window::{show_main_window, show_translation_window};
use crate::AppState;

use tauri::{Emitter, State, WebviewWindow};

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

pub fn get_word_under_cursor(
    app_state: State<'_, AppState>,
    window: WebviewWindow,
    custom_x: i32,
    custom_y: i32,
) {
    #[cfg(target_os = "macos")]
    {
        let capture_res = capture_around_cursor(200, 40);
        let Ok(img) = capture_res else { return };
        let words_res = recognize_words(&img, &app_state.ocr_state);
        let Ok(words) = words_res else { return };
        if let Some(word) = select_word(&words) {
            if !word.is_empty() {
                show_main_window(&window, custom_x, custom_y);
                window.emit("from-cursor", word).ok();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let capture_res = capture_around_cursor(200, 40);
        let Ok(img) = capture_res else { return };
        let words_res = recognize_words(&img, &app_state.ocr_state);
        let Ok(words) = words_res else { return };
        if let Some(word) = select_word(&words) {
            if !word.is_empty() {
                show_main_window(&window, 0, 0);
                window.emit("from-cursor", word).ok();
            }
        }
    }
}
