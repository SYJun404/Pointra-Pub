use crate::utils::ocr_common::{self, NormalizedRect, WordBox};
use objc2::{rc::Retained, AnyThread};
use objc2_core_foundation::CGRect;
use objc2_foundation::{NSArray, NSData, NSDictionary};
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, VNRecognizedTextObservation, VNRequest,
    VNRequestTextRecognitionLevel,
};
use std::sync::{Arc, Mutex};

pub use crate::utils::ocr_common::{select_sentence, select_word};

// ── Tauri State：缓存启动时初始化的 OCR 配置 ──────────────
pub struct OcrState {
    /// 复用的 VNRecognizeTextRequest（已配置好语言和精度）
    pub request: Mutex<Retained<VNRecognizeTextRequest>>,
}

// SAFETY: VNRecognizeTextRequest 在 Mutex 保护下跨线程使用
unsafe impl Send for OcrState {}
unsafe impl Sync for OcrState {}

impl OcrState {
    /// 应在 Tauri setup() 中调用一次，初始化并缓存 request。
    /// 内部会用一张小白图做一次预热推理，把 CoreML 模型加载
    /// 提前到启动阶段，消除首次识别的 1-3 秒延迟。
    pub fn new() -> Arc<Self> {
        let request = unsafe { build_english_request() };
        let state = Arc::new(Self {
            request: Mutex::new(request),
        });
        // clone 一份引用交给后台预热线程，不阻塞 UI 启动
        warmup_async(Arc::clone(&state));
        state
    }
}

/// 构建支持中英混排的 VNRecognizeTextRequest，只执行一次。
unsafe fn build_english_request() -> Retained<VNRecognizeTextRequest> {
    let request = VNRecognizeTextRequest::new();
    request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
    request.setUsesLanguageCorrection(true);
    request.setAutomaticallyDetectsLanguage(true);
    request
}

/// 在独立线程里跑一次空白图推理，预热 CoreML / ANE
fn warmup_async(state: Arc<OcrState>) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = warmup_once(&state) {
            eprintln!("[MAC OCR warmup] 预热失败（不影响功能）: {e}");
        } else {
            eprintln!("[MAC OCR warmup] 预热完成");
        }
    });
}

/// 构造一张 64×64 纯白 PNG，跑一次完整推理以触发模型加载
fn warmup_once(state: &OcrState) -> anyhow::Result<()> {
    use image::{ImageBuffer, Luma};
    let img = ImageBuffer::<Luma<u8>, _>::from_pixel(64, 64, Luma([255u8]));
    let dyn_img = image::DynamicImage::ImageLuma8(img);
    // 结果丢弃，只需模型完成一次完整推理
    let _ = recognize_words(&dyn_img, state)?;
    Ok(())
}

/// 将 objc2::CGRect 转换为共用的 NormalizedRect
fn nr_from_cg(rect: CGRect) -> NormalizedRect {
    NormalizedRect {
        x: rect.origin.x,
        y: rect.origin.y,
        w: rect.size.width,
        h: rect.size.height,
    }
}

// ── Vision OCR（使用缓存 State）──────────────────────────
pub fn recognize_words(
    img: &image::DynamicImage,
    state: &OcrState,
) -> anyhow::Result<Vec<WordBox>> {
    let mut png: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)?;

    unsafe {
        let ns_data = NSData::with_bytes(&png);

        // 从 State 取出已配置好的 request，无需重建
        let request = state
            .request
            .lock()
            .map_err(|_| anyhow::anyhow!("OcrState mutex poisoned"))?;

        // 向上转型 VNRecognizeTextRequest → VNRequest
        // clone retained pointer 以构造 NSArray，不移动 request
        let as_vn: Retained<VNRequest> = {
            let ptr = &**request as *const VNRecognizeTextRequest;
            // SAFETY: VNRecognizeTextRequest 继承自 VNRequest
            Retained::retain(ptr as *mut VNRequest)
                .ok_or_else(|| anyhow::anyhow!("retain VNRequest failed"))?
        };
        let requests = NSArray::from_retained_slice(&[as_vn]);

        let handler = VNImageRequestHandler::initWithData_options(
            VNImageRequestHandler::alloc(),
            &ns_data,
            &NSDictionary::new(),
        );

        handler
            .performRequests_error(&requests)
            .map_err(|e| anyhow::anyhow!("Vision 执行失败: {:?}", e))?;

        let request_ptr = &**request as *const VNRecognizeTextRequest;
        let results: Retained<NSArray<VNRecognizedTextObservation>> = (*request_ptr)
            .results()
            .ok_or_else(|| anyhow::anyhow!("OCR 无结果"))?;

        let mut words = Vec::new();

        for obs in results.iter() {
            let candidates = obs.topCandidates(1);
            let candidate = match candidates.firstObject() {
                Some(c) => c,
                None => continue,
            };

            let text: String = candidate.string().to_string();
            let text_box_cg: CGRect = obs.boundingBox();
            let text_box = nr_from_cg(text_box_cg);

            // 脚本感知分词：中英混排行里拉丁段做 CamelCase 拆分
            for byte_range in ocr_common::script_aware_ranges(&text) {
                let token = &text[byte_range.clone()];
                if !ocr_common::contains_letter(token) {
                    continue;
                }

                // 尝试 Vision 精确 BBox（UTF-16 偏移）
                let precise: Option<NormalizedRect> = {
                    // 纯 ASCII 时 UTF-16 偏移 == 字节偏移，直接用 len()
                    let utf16_start = if text[..byte_range.start].is_ascii() {
                        byte_range.start
                    } else {
                        text[..byte_range.start].encode_utf16().count()
                    };
                    let utf16_len = if text[byte_range.start..byte_range.end].is_ascii() {
                        byte_range.end - byte_range.start
                    } else {
                        text[byte_range.start..byte_range.end]
                            .encode_utf16()
                            .count()
                    };
                    let ns_range = objc2_foundation::NSRange {
                        location: utf16_start,
                        length: utf16_len,
                    };
                    candidate
                        .boundingBoxForRange_error(ns_range)
                        .ok()
                        .map(|obs| nr_from_cg(obs.boundingBox()))
                };

                let fallback = ocr_common::fallback_box(text_box, &text, &byte_range);
                let is_sub = byte_range != (0..text.len());

                if let Some(b) = ocr_common::resolved_box(precise, fallback, text_box, is_sub) {
                    words.push(WordBox {
                        text: token.to_string(),
                        x: b.x,
                        y: b.y,
                        w: b.w,
                        h: b.h,
                    });
                }
            }
        }

        Ok(words)
    }
}
