use tauri::{AppHandle, Manager, WebviewWindow};

use tauri_nspanel::{
    tauri_panel, CollectionBehavior, PanelLevel, StyleMask, TrackingAreaOptions, WebviewWindowExt,
};

tauri_panel! {
    panel!(BasicPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
        with: {
            // Enable mouse tracking for the panel's content view
            // This allows the panel to receive mouse events even when not key/active
            tracking_area: {
                options: TrackingAreaOptions::new()
                    .active_always()           // Track mouse even when app is not active
                    .mouse_entered_and_exited() // Get notified when mouse enters/exits
                    .mouse_moved(),             // Track mouse movement
                auto_resize: true               // Resize tracking area with window
            }
        }
    })
}

pub fn apply_squircle_corners<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>, radius: f64) {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl};

    let Ok(ns_win) = window.ns_window() else {
        return;
    };

    unsafe {
        let ns_win = ns_win as id;
        let content_view: id = msg_send![ns_win, contentView];

        if content_view != nil {
            let _: () = msg_send![content_view, setWantsLayer: true];

            let layer: id = msg_send![content_view, layer];
            if layer != nil {
                let _: () = msg_send![layer, setCornerRadius: radius];
                let _: () = msg_send![layer, setMasksToBounds: true];

                let continuous = NSString::alloc(nil).init_str("continuous");
                let _: () = msg_send![layer, setCornerCurve: continuous];
            }
        }
    }
}

pub fn apply_window_effects(app_handle: AppHandle) {
    {
        let web_window: WebviewWindow = app_handle.get_webview_window("main").unwrap();

        apply_squircle_corners(&web_window, 16.0);

        // apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
        //     .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

        // 打开调试工具
        // web_window.open_devtools();

        let panel = web_window.to_panel::<BasicPanel>().unwrap();

        // Set the window to float level
        panel.set_level(PanelLevel::Floating.value());

        // Ensures the panel cannot activate the app
        panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

        // Allows the panel to:
        // - display on the same space as the full screen window
        // - join all spaces
        panel.set_collection_behavior(
            CollectionBehavior::new()
                .full_screen_auxiliary()
                .can_join_all_spaces()
                .into(),
        );
    }
}
