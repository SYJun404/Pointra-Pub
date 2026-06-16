use crate::utils::ocr_common::{self, NormalizedRect, WordBox};
use std::sync::{Arc, Mutex};
use windows::Globalization::Language;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Graphics::Imaging::BitmapPixelFormat;
use windows::Graphics::Imaging::SoftwareBitmap;
use windows::Media::Ocr::OcrEngine;
use windows::Media::Ocr::OcrResult;
use windows::Storage::Streams::DataWriter;
use windows::Storage::Streams::InMemoryRandomAccessStream;

pub use crate::utils::ocr_common::{select_sentence, select_word};

// ── Tauri State ───────────────────────────────────────────
pub struct OcrState {
    /// 复用的 OcrEngine（已配置好语言）
    pub engine: Mutex<OcrEngine>,
}

unsafe impl Send for OcrState {}
unsafe impl Sync for OcrState {}

impl OcrState {
    /// 应在 Tauri setup() 中调用一次，初始化并缓存 OcrEngine。
    /// 内部会用一张小白图做一次预热推理。
    pub fn new() -> Arc<Self> {
        let engine = new_engine().unwrap_or_else(|_| {
            OcrEngine::TryCreateFromUserProfileLanguages().expect("Windows OCR 初始化失败")
        });
        let state = Arc::new(Self {
            engine: Mutex::new(engine),
        });
        warmup_async(Arc::clone(&state));
        state
    }
}

fn new_engine() -> windows::core::Result<OcrEngine> {
    let lang = Language::CreateLanguage(&windows::core::HSTRING::from("en"))?;
    match OcrEngine::TryCreateFromLanguage(&lang) {
        Ok(engine) => Ok(engine),
        Err(_) => OcrEngine::TryCreateFromUserProfileLanguages(),
    }
}

fn warmup_async(state: Arc<OcrState>) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = warmup_once(&state) {
            eprintln!("[WIN OCR warmup] 预热失败（不影响功能）: {e}");
        } else {
            eprintln!("[WIN OCR warmup] 预热完成");
        }
    });
}

fn warmup_once(state: &OcrState) -> anyhow::Result<()> {
    use image::{ImageBuffer, Luma};
    let img = image::DynamicImage::ImageLuma8(ImageBuffer::<Luma<u8>, _>::from_pixel(
        64,
        64,
        Luma([255u8]),
    ));
    let _ = recognize_words(&img, state)?;
    Ok(())
}

// ── 图像预处理（提升 Windows OCR 准确率） ──────────────────

/// 为 OCR 做图像预处理：小图放大 + 灰度化 + 自动对比度 + 轻度锐化
fn preprocess_for_ocr(img: &image::DynamicImage) -> image::DynamicImage {
    let (w, h) = (img.width(), img.height());
    let min_dim = w.min(h);

    // 1. 小图自动放大（Windows OCR 对 < 15px 的文字效果很差）
    //    放大后最小边 ≥ 400px，上限 3x，用 Lanczos3 保边缘清晰
    let scaled = if min_dim < 400 {
        let scale = (400.0 / min_dim as f64).min(3.0);
        let new_w = (w as f64 * scale).round() as u32;
        let new_h = (h as f64 * scale).round() as u32;
        image::DynamicImage::ImageRgba8(image::imageops::resize(
            img,
            new_w,
            new_h,
            image::imageops::FilterType::Lanczos3,
        ))
    } else {
        img.clone()
    };

    // 2. 转灰度（OCR 引擎在灰度图上更稳定）
    let gray = scaled.grayscale();

    // 3. 自动对比度拉伸，让文字和背景分离更清晰
    let contrasted = auto_contrast(&gray);

    // 4. Unsharp Mask 轻度锐化，让字符边缘更锐利
    let sharpened = unsharp_mask(&contrasted, 1.2, 0.5);

    sharpened
}

/// 自动对比度拉伸：裁剪两端 0.5% outlier 后线性映射到 0..255
fn auto_contrast(img: &image::DynamicImage) -> image::DynamicImage {
    let gray = img.to_luma8();
    let mut histogram = [0u32; 256];
    for p in gray.pixels() {
        histogram[p.0[0] as usize] += 1;
    }

    let total = (gray.width() * gray.height()) as u32;
    let clip = (total as f64 * 0.005).max(1.0) as u32; // 两端各 0.5%

    let mut acc = 0u32;
    let mut min_val = 0u8;
    for (i, &count) in histogram.iter().enumerate() {
        acc += count;
        if acc > clip {
            min_val = i as u8;
            break;
        }
    }

    acc = 0;
    let mut max_val = 255u8;
    for (i, &count) in histogram.iter().enumerate().rev() {
        acc += count;
        if acc > clip {
            max_val = i as u8;
            break;
        }
    }

    if max_val <= min_val || (max_val - min_val) < 10 {
        return img.clone(); // 已经是高对比度，跳过
    }

    let scale = 255.0 / (max_val as f64 - min_val as f64);
    let mut out = image::GrayImage::new(gray.width(), gray.height());
    for (x, y, p) in gray.enumerate_pixels() {
        let v = p.0[0];
        let new_v = if v <= min_val {
            0
        } else if v >= max_val {
            255
        } else {
            ((v as f64 - min_val as f64) * scale).round() as u8
        };
        out.put_pixel(x, y, image::Luma([new_v]));
    }

    image::DynamicImage::ImageLuma8(out)
}

/// 手动高斯模糊（image 0.25 的 imageops 不提供 gaussian_blur）
fn gaussian_blur_gray(img: &image::GrayImage, sigma: f32) -> image::GrayImage {
    let (w, h) = img.dimensions();
    if sigma <= 0.0 || (w < 3 && h < 3) {
        return img.clone();
    }

    // 计算 kernel 半径 (sigma * 3 通常足够)
    let radius = (sigma * 3.0).ceil() as i32;
    let size = (2 * radius + 1) as usize;

    // 1D Gaussian kernel
    let mut kernel = Vec::with_capacity(size);
    let mut sum = 0.0f32;
    for i in -(radius as i32)..=radius {
        let v = (-(i * i) as f32 / (2.0 * sigma * sigma)).exp();
        kernel.push(v);
        sum += v;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    // 水平方向模糊 -> 临时 buffer
    let mut tmp = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &k) in kernel.iter().enumerate() {
                let sx = (x as i32 + ki as i32 - radius).clamp(0, w as i32 - 1) as u32;
                let p = img.get_pixel(sx, y);
                acc += p.0[0] as f32 * k;
            }
            tmp[(y * w + x) as usize] = acc.round() as u8;
        }
    }

    // 垂直方向模糊 -> 输出
    let mut out = image::GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &k) in kernel.iter().enumerate() {
                let sy = (y as i32 + ki as i32 - radius).clamp(0, h as i32 - 1) as u32;
                let v = tmp[(sy * w + x) as usize];
                acc += v as f32 * k;
            }
            out.put_pixel(x, y, image::Luma([acc.round() as u8]));
        }
    }

    out
}

/// Unsharp Mask：original + (original - blurred) * amount
fn unsharp_mask(img: &image::DynamicImage, sigma: f32, amount: f32) -> image::DynamicImage {
    let gray = img.to_luma8();
    let blurred = gaussian_blur_gray(&gray, sigma);

    let mut out = image::GrayImage::new(gray.width(), gray.height());
    for (x, y, &image::Luma([orig])) in gray.enumerate_pixels() {
        let image::Luma([blur]) = blurred.get_pixel(x, y);
        let diff = orig as i16 - *blur as i16;
        let sharpened = (orig as f32 + diff as f32 * amount).clamp(0.0, 255.0) as u8;
        out.put_pixel(x, y, image::Luma([sharpened]));
    }

    image::DynamicImage::ImageLuma8(out)
}

/// DynamicImage 编码为 PNG → InMemoryRandomAccessStream → BitmapDecoder → SoftwareBitmap
fn to_software_bitmap(img: &image::DynamicImage) -> anyhow::Result<SoftwareBitmap> {
    // 1. 编码为 PNG
    let mut png: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)?;

    // 2. 写入 InMemoryRandomAccessStream
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(&png)?;
    writer.StoreAsync()?.get()?;
    writer.FlushAsync()?.get()?;
    stream.Seek(0)?;

    // 3. 创建 BitmapDecoder
    // windows 0.51: CreateAsync 是 BitmapDecoder 的静态方法
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;

    // 4. 获取 SoftwareBitmap
    let sb = decoder.GetSoftwareBitmapAsync()?.get()?;

    // 5. 确保 BGRA8 格式
    if sb.BitmapPixelFormat()? != BitmapPixelFormat::Bgra8 {
        Ok(SoftwareBitmap::Convert(&sb, BitmapPixelFormat::Bgra8)?)
    } else {
        Ok(sb)
    }
}

/// 识别图片中的英文文字，返回带归一化坐标的单词列表。
pub fn recognize_words(
    img: &image::DynamicImage,
    state: &OcrState,
) -> anyhow::Result<Vec<WordBox>> {
    // ── 图像预处理（提升小字/模糊/低对比度场景的准确率） ──
    let processed = preprocess_for_ocr(img);
    let (img_w, img_h) = (processed.width(), processed.height());

    // ── 创建 SoftwareBitmap ────────────────────────────
    let sb = to_software_bitmap(&processed)?;

    // ── OCR 识别 ─────────────────────────────────────────
    let engine = state
        .engine
        .lock()
        .map_err(|_| anyhow::anyhow!("OcrState mutex poisoned"))?;

    let ocr_result: OcrResult = engine.RecognizeAsync(&sb)?.get()?;
    let lines = ocr_result.Lines()?;

    let mut words = Vec::new();

    for line in lines.into_iter() {
        // Windows OCR 提供单词级别的 BoundingRect，比行级更精确
        let ocr_words = line.Words()?;
        for ocr_word in ocr_words.into_iter() {
            let word_text = ocr_word.Text()?.to_string();
            if word_text.trim().is_empty() {
                continue;
            }

            // 对每个词做脚本感知分词（处理 CamelCase 混合文本）
            for byte_range in ocr_common::script_aware_ranges(&word_text) {
                let token = &word_text[byte_range.clone()];
                if !ocr_common::contains_letter(token) {
                    continue;
                }

                let word_rect = ocr_word.BoundingRect()?;
                // 归一化
                let wx = word_rect.X as f64 / img_w as f64;
                let wy = word_rect.Y as f64 / img_h as f64;
                let ww = word_rect.Width as f64 / img_w as f64;
                let wh = word_rect.Height as f64 / img_h as f64;
                let word_normal = NormalizedRect {
                    x: wx,
                    y: wy,
                    w: ww,
                    h: wh,
                };

                let is_sub = byte_range != (0..word_text.len());

                // 子词（CamelCase 拆分出的部分）用 fallback 估算
                if is_sub {
                    let fb = ocr_common::fallback_box(word_normal, &word_text, &byte_range);
                    if let Some(b) = ocr_common::resolved_box(None, fb, word_normal, is_sub) {
                        words.push(WordBox {
                            text: token.to_string(),
                            x: b.x,
                            y: b.y,
                            w: b.w,
                            h: b.h,
                        });
                    }
                } else {
                    // 完整词直接使用 OCR 提供的精确 BoundingRect
                    words.push(WordBox {
                        text: token.to_string(),
                        x: wx,
                        y: wy,
                        w: ww,
                        h: wh,
                    });
                }
            }
        }
    }

    Ok(words)
}
