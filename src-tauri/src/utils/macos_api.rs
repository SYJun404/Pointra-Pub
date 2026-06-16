#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {

    fn AXIsProcessTrusted() -> bool;

}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;

}

/// 检查辅助功能权限
pub unsafe fn check_accessibility() -> bool {
    AXIsProcessTrusted()
}

/// 检查屏幕录制权限
pub unsafe fn check_screen_recording() -> bool {
    CGPreflightScreenCaptureAccess()
}
