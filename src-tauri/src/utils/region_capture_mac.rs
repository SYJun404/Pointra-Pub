use anyhow::Context;
use std::{path::PathBuf, process::Command};

struct ScreenshotFile(PathBuf);

impl Drop for ScreenshotFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// 系统原生拖选界面支持多显示器，Esc 取消；只将框选像素送入 OCR。
pub(super) fn capture_selected_region() -> anyhow::Result<Option<image::DynamicImage>> {
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
