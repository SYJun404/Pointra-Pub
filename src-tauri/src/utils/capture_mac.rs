use anyhow::Context;
use core_graphics::display::CGDisplay;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use image::DynamicImage;

fn display_under_mouse(mouse: CGPoint) -> Option<(CGDisplay, CGRect)> {
    let ids = CGDisplay::active_displays().ok()?;
    for id in ids {
        let display = CGDisplay::new(id);
        let bounds = display.bounds();
        if bounds.contains(&mouse) {
            return Some((display, bounds));
        }
    }
    None
}

pub fn capture_at_position(
    position: (i32, i32),
    capture_w: i32,
    capture_h: i32,
) -> anyhow::Result<DynamicImage> {
    let mouse_pos = CGPoint::new(position.0 as f64, position.1 as f64);

    // 找到鼠标所在显示器
    let (display, disp_bounds) =
        display_under_mouse(mouse_pos).context("mouse is not on any active display")?;

    let global_rect = CGRect::new(
        &CGPoint::new(
            mouse_pos.x - capture_w as f64 / 2.0,
            mouse_pos.y - capture_h as f64 / 2.0,
        ),
        &CGSize::new(capture_w as f64, capture_h as f64),
    );

    let local_rect = CGRect::new(
        &CGPoint::new(
            global_rect.origin.x - disp_bounds.origin.x,
            global_rect.origin.y - disp_bounds.origin.y,
        ),
        &global_rect.size,
    );

    // 从对应显示器截图
    let cg_image = display
        .image_for_rect(local_rect)
        .context("capture failed")?;

    // CGImage → DynamicImage
    let (img_w, img_h) = (cg_image.width(), cg_image.height());
    let src_stride = cg_image.bytes_per_row();
    let raw_data = cg_image.data();
    let raw_ref = raw_data.bytes();

    // macOS CGImage 像素格式为 BGRA（kCGBitmapByteOrder32Little），
    // 且 bytes_per_row 可能包含 stride 填充。逐行复制并转为 RGBA。
    let row_bytes = (img_w * 4) as usize;
    let mut rgba_bytes: Vec<u8> = Vec::with_capacity((img_h as usize) * row_bytes);

    for y in 0..img_h as usize {
        let row = &raw_ref[y * src_stride..y * src_stride + row_bytes];
        for px in row.chunks(4) {
            rgba_bytes.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
    }

    let img = image::RgbaImage::from_raw(img_w as u32, img_h as u32, rgba_bytes)
        .map(DynamicImage::ImageRgba8)
        .context("图像字节长度不匹配")?;

    Ok(img)
}
