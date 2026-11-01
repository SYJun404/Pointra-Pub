use crate::{
    utils::{
        define_config::{keycode_from_str, AppConfig},
        get_text::{get_data_from_selected_text, get_word_under_cursor},
        show_window::{show_input_window, show_main_window_at_position},
    },
    AppState,
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use mouse_position::mouse_position::Mouse as MousePosition;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// A release or a newer press invalidates all results from the previous press.
#[derive(Default)]
struct PointSession(AtomicU64);

impl PointSession {
    fn advance(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel) + 1
    }

    fn is_current(&self, generation: u64) -> bool {
        self.0.load(Ordering::Acquire) == generation
    }
}

#[derive(Clone, Copy)]
struct PointRequest {
    generation: u64,
    position: (i32, i32),
    started: Instant,
}

/// Route every outcome through the app-level listener, even on the search page.
fn publish_cursor_state(
    app: &AppHandle,
    session: &Arc<PointSession>,
    request: PointRequest,
    feedback: Value,
) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let session = Arc::clone(session);
    let _ = app.run_on_main_thread(move || {
        if !session.is_current(request.generation) {
            return;
        }
        let _ = window.emit(
            "win-event",
            json!({"type": "cursor-state", "data": feedback}),
        );
        show_main_window_at_position(&window, request.position, 0, 0);
        if std::env::var_os("POINTRA_TRACE_CURSOR").is_some() {
            eprintln!("[Cursor] press-to-feedback={:?}", request.started.elapsed());
        }
    });
}

/// Poll independently of the serial capture/OCR worker, including during cold starts.
pub fn init_point_listener(app_handle: AppHandle) {
    let session = Arc::new(PointSession::default());
    let (sender, receiver) = mpsc::channel::<PointRequest>();
    let worker_app = app_handle.clone();
    let worker_session = Arc::clone(&session);
    thread::spawn(move || {
        while let Ok(mut request) = receiver.recv() {
            // Coalesce presses during a slow inference; never run old queued captures.
            for newer in receiver.try_iter() {
                request = newer;
            }
            if !worker_session.is_current(request.generation) {
                continue;
            }
            let state = worker_app.state::<AppState>();
            let result = get_word_under_cursor(&state, request.position, || {
                publish_cursor_state(
                    &worker_app,
                    &worker_session,
                    request,
                    json!({"status": "loading"}),
                );
            });
            let feedback = match result {
                Ok(Some(word)) => json!({"status": "word", "word": word}),
                Ok(None) => json!({"status": "empty"}),
                Err(error) => {
                    eprintln!("[Cursor] {error:#}");
                    json!({"status": "error", "message": "取词失败，请检查屏幕录制权限，或稍后重试。"})
                }
            };
            publish_cursor_state(&worker_app, &worker_session, request, feedback);
        }
    });

    // This lifetime-long loop must not occupy a Tokio async worker.
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut last_state = false;
        loop {
            let current_key = {
                let state = app_handle.state::<AppState>();
                let config = state.config.lock().unwrap();
                keycode_from_str(&config.point_key).unwrap_or(Keycode::F2)
            };
            let is_pressed = device_state.get_keys().contains(&current_key);
            if is_pressed != last_state {
                last_state = is_pressed;
                let generation = session.advance();
                if is_pressed {
                    if let MousePosition::Position { x, y } = MousePosition::get_mouse_position() {
                        let _ = sender.send(PointRequest {
                            generation,
                            position: (x, y),
                            started: Instant::now(),
                        });
                    }
                } else {
                    let app = app_handle.clone();
                    let session = Arc::clone(&session);
                    let _ = app_handle.run_on_main_thread(move || {
                        let state = app.state::<AppState>();
                        if session.is_current(generation)
                            && !state.window_locked.load(Ordering::Relaxed)
                        {
                            if let Some(window) = app.get_webview_window("main") {
                                app.state::<crate::commands::translate::TranslationTasks>()
                                    .abort_all();
                                let _ = window.emit("window-will-hide", ());
                                let _ = window.hide();
                            }
                        }
                    });
                }
            }
            thread::sleep(Duration::from_millis(60));
        }
    });
}

/// 初始化快捷键
pub fn init_shortcuts<R: Runtime>(app: &AppHandle<R>) {
    let shortcut_manager = app.global_shortcut();
    let app_state = app.state::<AppState>();
    let config = app_state.config.lock().unwrap();

    let shortcut_select_text =
        Shortcut::new(Some(config.select_text_modifiers), config.select_text_code);
    let shortcut_show_input =
        Shortcut::new(Some(config.search_win_modifiers), config.search_win_code);

    let _ = shortcut_manager.register(shortcut_select_text);
    let _ = shortcut_manager.register(shortcut_show_input);
    #[cfg(target_os = "macos")]
    {
        let shortcut = Shortcut::new(
            Some(config.region_capture_modifiers),
            config.region_capture_code,
        );
        if let Err(error) = shortcut_manager.register(shortcut) {
            eprintln!("[Region OCR] 注册框选快捷键失败: {error}");
        }
    }
}

/// 处理快捷键事件
pub fn handle_shortcut_event(app: &AppHandle, shortcut: &Shortcut) {
    let app_state = app.state::<AppState>();
    let config = app_state.config.lock().unwrap();

    if let Some(win) = app.get_webview_window("main") {
        if shortcut.matches(config.select_text_modifiers, config.select_text_code) {
            get_data_from_selected_text(win);
        } else if shortcut.matches(config.search_win_modifiers, config.search_win_code) {
            show_input_window(win);
        }
        #[cfg(target_os = "macos")]
        if shortcut.matches(config.region_capture_modifiers, config.region_capture_code) {
            crate::utils::region_capture_mac::start_region_capture(app.clone());
        }
    }
}

/// 更新全局快捷键
pub fn restart_shortcuts<R: Runtime>(app: &AppHandle<R>, new_config: AppConfig) {
    // 设置改变时移除旧组合，避免旧按键继续占用或重复注册。
    let _ = app.global_shortcut().unregister_all();
    {
        let app_state = app.state::<AppState>();
        let mut config = app_state.config.lock().unwrap();
        *config = new_config;
    }

    init_shortcuts(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_during_slow_ocr_does_not_publish_a_late_result() {
        let session = Arc::new(PointSession::default());
        let press = session.advance();
        let (started_tx, started_rx) = mpsc::channel();
        let (finish_tx, finish_rx) = mpsc::channel();
        let worker_session = Arc::clone(&session);
        let worker = thread::spawn(move || {
            started_tx.send(()).unwrap();
            // Model a native inference that cannot be cancelled in progress.
            finish_rx.recv().unwrap();
            worker_session.is_current(press)
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        session.advance(); // The listener processes release while OCR is blocked.
        finish_tx.send(()).unwrap();
        assert!(
            !worker.join().unwrap(),
            "late OCR must not publish after release"
        );
    }

    #[test]
    fn new_press_invalidates_old_result_and_queued_hide() {
        let session = PointSession::default();
        let old_press = session.advance();
        let old_hide = session.advance();
        let new_press = session.advance();
        assert!(!session.is_current(old_press));
        assert!(!session.is_current(old_hide));
        assert!(session.is_current(new_press));
    }
}
