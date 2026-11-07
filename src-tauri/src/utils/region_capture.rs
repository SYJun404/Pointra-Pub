use super::show_window::{show_main_window, show_translation_window};
use crate::AppState;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "macos")]
use super::{ocr_mac::recognize_region_text, region_capture_mac::capture_selected_region};
#[cfg(target_os = "windows")]
use super::{ocr_win::recognize_region_text, region_capture_win::capture_selected_region};

static CAPTURING: AtomicBool = AtomicBool::new(false);

struct CaptureGuard;

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        CAPTURING.store(false, Ordering::Release);
    }
}

pub fn start_region_capture(app: AppHandle) {
    if CAPTURING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let guard = CaptureGuard;
    // 原生框选会等待用户操作，OCR 同样是阻塞调用，均放到阻塞线程池。
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        let was_visible = window.is_visible().unwrap_or(false);
        app.state::<crate::commands::translate::TranslationTasks>()
            .abort_all();
        let _ = window.emit("window-will-hide", ());
        let _ = window.hide();

        let result = (|| -> anyhow::Result<Option<String>> {
            let Some(image) = capture_selected_region()? else {
                return Ok(None);
            };
            let state = app.state::<AppState>();
            let text = recognize_region_text(&image, &state.ocr_state)?;
            anyhow::ensure!(!text.trim().is_empty(), "选中区域没有识别到文字");
            Ok(Some(text))
        })();

        match result {
            Ok(Some(text)) => {
                show_translation_window(&window, text, 16, 18);
            }
            Ok(None) => {
                if was_visible {
                    let _ = window.show();
                }
            }
            Err(error) => {
                eprintln!("[Region OCR] {error:#}");
                show_main_window(&window, 16, 18);
                let _ = window.emit(
                    "win-event",
                    json!({"type": "region-error", "data": format!("{error:#}")}),
                );
            }
        }
    });
}
