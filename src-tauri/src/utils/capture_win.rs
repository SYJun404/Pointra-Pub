use anyhow::Context;
use image::{DynamicImage, Rgb, RgbImage};
use screenshots::Screen;

/// Capture at the press position and return the cursor offset in the captured image.
pub fn capture_at_position(
    (x, y): (i32, i32),
    width: i32,
    height: i32,
) -> anyhow::Result<(DynamicImage, (i32, i32))> {
    let screen = Screen::from_point(x, y).context("无法找到鼠标所在的显示器")?;
    let display = screen.display_info;
    let cursor = (x - display.x, y - display.y);
    let left = (cursor.0 - width / 2).max(0);
    let top = (cursor.1 - height / 2).max(0);
    let right = (cursor.0 + width / 2).min(display.width as i32);
    let bottom = (cursor.1 + height / 2).min(display.height as i32);
    let pixels = screen
        .capture_area_ignore_area_check(left, top, (right - left) as u32, (bottom - top) as u32)
        .context("屏幕截图失败")?;
    // GDI does not provide meaningful alpha for desktop captures.
    let image = RgbImage::from_fn(pixels.width(), pixels.height(), |x, y| {
        let [r, g, b, _] = pixels.get_pixel(x, y).0;
        Rgb([r, g, b])
    });
    Ok((
        DynamicImage::ImageRgb8(image),
        (cursor.0 - left, cursor.1 - top),
    ))
}
