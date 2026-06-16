use anyhow::Context;
use image::DynamicImage;

/// 在光标周围截取指定区域
/// - `capture_w`: 截图宽度（像素）
/// - `capture_h`: 截图高度（像素）
pub fn capture_around_cursor(capture_w: i32, capture_h: i32) -> anyhow::Result<DynamicImage> {
    // 1. 获取鼠标位置（屏幕坐标）
    let mouse = mouse_position::mouse_position::Mouse::get_mouse_position();
    let (mouse_x, mouse_y) = match mouse {
        mouse_position::mouse_position::Mouse::Position { x, y } => (x, y),
        mouse_position::mouse_position::Mouse::Error => anyhow::bail!("获取鼠标位置失败"),
    };

    // 2. 找到鼠标所在的显示器
    let screen =
        screenshots::Screen::from_point(mouse_x, mouse_y).context("无法找到鼠标所在的显示器")?;

    // 3. 计算截取区域（以光标为中心）
    let cap_x = mouse_x - capture_w / 2;
    let cap_y = mouse_y - capture_h / 2;

    // 4. 截取区域图像（screenshots 返回 image 0.24 的类型，需手动转换）
    let rgba_img = screen
        .capture_area(cap_x, cap_y, capture_w as u32, capture_h as u32)
        .context("屏幕截图失败")?;

    let (w, h) = rgba_img.dimensions();
    let raw: Vec<u8> = rgba_img.as_raw().to_vec();

    let dyn_img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(w, h, raw).context("图像字节长度不匹配")?,
    );

    Ok(dyn_img)
}
