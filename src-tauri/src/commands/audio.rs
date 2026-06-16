use crate::AppState;
use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AudioState {
    pub sink: Arc<Mutex<Option<Sink>>>,
    pub _stream: Arc<Mutex<Option<OutputStream>>>,
}

unsafe impl Send for AudioState {}
unsafe impl Sync for AudioState {}

impl AudioState {
    pub fn new() -> Self {
        AudioState {
            sink: Arc::new(Mutex::new(None)),
            _stream: Arc::new(Mutex::new(None)),
        }
    }
}

/// 播放 URL 音频
#[tauri::command]
pub async fn play_phonetic_url(url: String, state: State<'_, AppState>) -> Result<(), String> {
    // 从state中获取用户设置的播放音量
    let volume = state.config.lock().unwrap().pronunciation_volume as f32 / 200.0;

    // 下载音频数据
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| format!("请求失败: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("读取数据失败: {}", e))?;
    let cursor = Cursor::new(bytes.to_vec());

    // 创建输出流
    let (stream, stream_handle) =
        OutputStream::try_default().map_err(|e| format!("音频设备初始化失败: {}", e))?;
    let sink = Sink::try_new(&stream_handle).map_err(|e| format!("创建 Sink 失败: {}", e))?;
    sink.set_volume(volume);

    // 解码音频
    let decoder = Decoder::new(cursor).map_err(|e| format!("解码音频失败: {}", e))?;
    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();

    // 1. 将所有采样点转换为 f32 格式并收集到内存
    let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

    // 2. 寻找当前音频的最大振幅（绝对值）
    let mut max_amplitude = 0.0f32;
    for &sample in &samples {
        let abs = sample.abs();
        if abs > max_amplitude {
            max_amplitude = abs;
        }
    }

    // 3. 统一分贝：进行峰值归一化
    // 目标最大振幅设为 0.8（保留 10% 裕度防止某些设备上产生破音/削波）
    let target_peak = 0.8f32;
    let normalized_samples = if max_amplitude > 0.0001 {
        let gain = target_peak / max_amplitude;
        // 对所有采样点乘以增益系数
        samples.into_iter().map(|s| s * gain).collect()
    } else {
        samples
    };

    // 4. 重建音频源并播放
    let source = SamplesBuffer::new(channels, sample_rate, normalized_samples);
    sink.append(source);

    // 保存到状态
    *state.audio_state.sink.lock().unwrap() = Some(sink);
    *state.audio_state._stream.lock().unwrap() = Some(stream);
    Ok(())
}
