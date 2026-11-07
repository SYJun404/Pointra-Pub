use anyhow::Context;
use image::{DynamicImage, Rgba, RgbaImage};
use screenshots::Screen;
use std::{cell::Cell, sync::OnceLock};
use windows::{
    core::{w, Error},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{
            Dwm::DwmFlush,
            Gdi::{
                BeginPaint, EndPaint, FillRect, FrameRect, GetStockObject, InvalidateRect,
                ScreenToClient, BLACK_BRUSH, HBRUSH, PAINTSTRUCT, WHITE_BRUSH,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            Input::KeyboardAndMouse::{
                RegisterHotKey, SetCapture, UnregisterHotKey, MOD_NOREPEAT, VK_ESCAPE,
            },
            WindowsAndMessaging::*,
        },
    },
};

const CANCEL_HOTKEY_ID: i32 = 1;

#[derive(Clone, Copy, Default)]
struct Selection {
    start: Option<POINT>,
    rect: Option<RECT>,
    closed: bool,
}

thread_local! {
    static SELECTION: Cell<Selection> = Cell::new(Selection::default());
}

fn screen_rect(screen: &Screen) -> RECT {
    let display = screen.display_info;
    RECT {
        left: display.x,
        top: display.y,
        right: display.x + display.width as i32,
        bottom: display.y + display.height as i32,
    }
}

fn intersection(a: RECT, b: RECT) -> Option<RECT> {
    let rect = RECT {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    };
    (rect.left < rect.right && rect.top < rect.bottom).then_some(rect)
}

pub(super) fn capture_selected_region() -> anyhow::Result<Option<DynamicImage>> {
    // Keep the overlay, monitor bounds, and captured pixels in the same coordinate space.
    let previous =
        unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    anyhow::ensure!(previous.0 != 0, "无法设置框选窗口的 DPI 模式");
    let result = capture_physical_region();
    unsafe { SetThreadDpiAwarenessContext(previous) };
    result
}

fn capture_physical_region() -> anyhow::Result<Option<DynamicImage>> {
    let screens = Screen::all().context("无法获取显示器列表")?;
    let bounds = screens
        .iter()
        .map(screen_rect)
        .reduce(|a, b| RECT {
            left: a.left.min(b.left),
            top: a.top.min(b.top),
            right: a.right.max(b.right),
            bottom: a.bottom.max(b.bottom),
        })
        .context("没有可截图的显示器")?;
    let Some(rect) = select_region(bounds)? else {
        return Ok(None);
    };
    if !screens
        .iter()
        .any(|screen| intersection(rect, screen_rect(screen)).is_some())
    {
        return Ok(None);
    }

    // Wait for the compositor after closing the overlay before reading the desktop.
    unsafe { DwmFlush() }.context("无法同步桌面画面")?;
    let mut image = RgbaImage::from_pixel(
        (rect.right - rect.left) as u32,
        (rect.bottom - rect.top) as u32,
        Rgba([255, 255, 255, 255]),
    );
    for screen in &screens {
        let monitor = screen_rect(screen);
        let Some(part) = intersection(rect, monitor) else {
            continue;
        };
        let pixels = screen
            .capture_area_ignore_area_check(
                part.left - monitor.left,
                part.top - monitor.top,
                (part.right - part.left) as u32,
                (part.bottom - part.top) as u32,
            )
            .context("无法截取选中区域")?;
        let mut pixels = RgbaImage::from_raw(pixels.width(), pixels.height(), pixels.into_raw())
            .context("截图像素数据无效")?;
        // The desktop bitmap's reserved alpha byte is undefined.
        for pixel in pixels.pixels_mut() {
            pixel[3] = 255;
        }
        image::imageops::replace(
            &mut image,
            &pixels,
            i64::from(part.left - rect.left),
            i64::from(part.top - rect.top),
        );
    }
    Ok(Some(DynamicImage::ImageRgba8(image)))
}

fn select_region(bounds: RECT) -> anyhow::Result<Option<RECT>> {
    static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();
    let class_name = w!("PointraRegionCapture");
    // All window operations, including message dispatch and destruction, use this worker.
    unsafe {
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        if CLASS_REGISTERED.get().is_none() {
            let class = WNDCLASSW {
                hInstance: instance,
                hCursor: LoadCursorW(None, IDC_CROSS)?,
                lpfnWndProc: Some(window_proc),
                lpszClassName: class_name,
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(Error::from_win32()).context("无法注册框选窗口");
            }
            // The shared capture guard serializes class registration and selection.
            let _ = CLASS_REGISTERED.set(());
        }
        SELECTION.set(Selection::default());
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name,
            w!("Pointra"),
            WS_POPUP,
            bounds.left,
            bounds.top,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            None,
            None,
            instance,
            None,
        );
        anyhow::ensure!(hwnd.0 != 0, "无法创建框选窗口: {}", Error::from_win32());
        let result = run_overlay(hwnd);
        if !SELECTION.get().closed {
            finish(hwnd, None);
        }
        Ok(result?.and_then(|rect| intersection(rect, bounds)))
    }
}

unsafe fn run_overlay(hwnd: HWND) -> anyhow::Result<Option<RECT>> {
    SetLayeredWindowAttributes(hwnd, COLORREF(0), 105, LWA_ALPHA).context("无法显示框选遮罩")?;
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetForegroundWindow(hwnd);
    if GetForegroundWindow() != hwnd {
        // Windows can deny activation until the user clicks the overlay.
        let _ = RegisterHotKey(hwnd, CANCEL_HOTKEY_ID, MOD_NOREPEAT, u32::from(VK_ESCAPE.0));
    }
    let mut message = MSG::default();
    while !SELECTION.get().closed {
        match GetMessageW(&mut message, None, 0, 0).0 {
            -1 => return Err(Error::from_win32()).context("无法读取框选输入"),
            0 => return Ok(None),
            _ => {
                DispatchMessageW(&message);
            }
        }
    }
    Ok(SELECTION.get().rect)
}

unsafe fn cursor_point() -> Option<POINT> {
    let mut point = POINT::default();
    GetCursorPos(&mut point).ok()?;
    Some(point)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut selection = SELECTION.get();
    match message {
        WM_LBUTTONDOWN => {
            selection.start = cursor_point();
            SELECTION.set(selection);
            SetCapture(hwnd);
        }
        WM_MOUSEMOVE | WM_LBUTTONUP if selection.start.is_some() => {
            selection.rect = selection
                .start
                .zip(cursor_point())
                .map(|(start, end)| RECT {
                    left: start.x.min(end.x),
                    top: start.y.min(end.y),
                    right: start.x.max(end.x),
                    bottom: start.y.max(end.y),
                });
            if message == WM_LBUTTONUP {
                finish(hwnd, selection.rect);
            } else {
                SELECTION.set(selection);
                let _ = InvalidateRect(hwnd, None, false);
            }
        }
        WM_KEYDOWN if wparam.0 == usize::from(VK_ESCAPE.0) => finish(hwnd, None),
        WM_HOTKEY if wparam.0 == CANCEL_HOTKEY_ID as usize => finish(hwnd, None),
        WM_ACTIVATE if wparam.0 & 0xffff == 0 => finish(hwnd, None),
        WM_CAPTURECHANGED if selection.start.is_some() => finish(hwnd, None),
        WM_RBUTTONDOWN | WM_CANCELMODE | WM_DISPLAYCHANGE | WM_CLOSE => finish(hwnd, None),
        WM_DESTROY => {
            let _ = UnregisterHotKey(hwnd, CANCEL_HOTKEY_ID);
            selection.closed = true;
            SELECTION.set(selection);
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut paint);
            FillRect(dc, &paint.rcPaint, HBRUSH(GetStockObject(BLACK_BRUSH).0));
            if let Some(rect) = selection.rect {
                let mut origin = POINT::default();
                let _ = ScreenToClient(hwnd, &mut origin);
                let outline = RECT {
                    left: rect.left + origin.x,
                    top: rect.top + origin.y,
                    right: rect.right + origin.x,
                    bottom: rect.bottom + origin.y,
                };
                FrameRect(dc, &outline, HBRUSH(GetStockObject(WHITE_BRUSH).0));
            }
            let _ = EndPaint(hwnd, &paint);
        }
        _ => return DefWindowProcW(hwnd, message, wparam, lparam),
    }
    LRESULT(0)
}

unsafe fn finish(hwnd: HWND, rect: Option<RECT>) {
    if !SELECTION.get().closed {
        SELECTION.set(Selection {
            start: None,
            rect,
            closed: true,
        });
        let _ = DestroyWindow(hwnd);
    }
}
