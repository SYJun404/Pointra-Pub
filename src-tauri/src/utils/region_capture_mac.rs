use super::{
    ocr_mac::recognize_region_text,
    show_window::{show_main_window, show_translation_window},
};
use crate::AppState;
use anyhow::Context;
use serde_json::json;
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};
use tauri::{AppHandle, Emitter, Manager};

static CAPTURING: AtomicBool = AtomicBool::new(false);

struct CaptureGuard;

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        CAPTURING.store(false, Ordering::Release);
    }
}

struct ScreenshotFile(PathBuf);

impl Drop for ScreenshotFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// 系统原生拖选界面支持多显示器，Esc 取消；只将框选像素送入 OCR。
fn capture_selected_region() -> anyhow::Result<Option<image::DynamicImage>> {
    let file = ScreenshotFile(
        std::env::temp_dir().join(format!("pointra-region-{}.png", uuid::Uuid::new_v4())),
    );
    let output = Command::new("/usr/sbin/screencapture")
        .args(["-i", "-s", "-x", "-t", "png"])
        .arg(&file.0)
        .output()
        .context("无法启动系统框选截图")?;

    if !file.0.exists() {
        let error = String::from_utf8_lossy(&output.stderr);
        let lower = error.to_ascii_lowercase();
        if output.status.success() || error.trim().is_empty() || lower.contains("cancel") {
            return Ok(None);
        }
        anyhow::bail!("截图失败，请检查屏幕录制权限：{}", error.trim());
    }
    let image = image::open(&file.0).context("无法读取框选截图")?;
    Ok(Some(image))
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
