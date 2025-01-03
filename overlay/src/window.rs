use windows::Win32::{
    Foundation::{
        BOOL,
        HWND,
    },
    Graphics::{
        Dwm::{
            DwmEnableBlurBehindWindow,
            DwmIsCompositionEnabled,
            DWM_BB_BLURREGION,
            DWM_BB_ENABLE,
            DWM_BLURBEHIND,
        },
        Gdi::{
            CreateRectRgn,
            DeleteObject,
        },
    },
    UI::WindowsAndMessaging::{
        SetWindowLongA,
        SetWindowLongPtrA,
        ShowWindow,
        GWL_EXSTYLE,
        GWL_STYLE,
        SW_SHOW,
        WS_CLIPSIBLINGS,
        WS_EX_APPWINDOW,
        WS_EX_LAYERED,
        WS_EX_TRANSPARENT,
        WS_VISIBLE,
    },
};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    raw_window_handle::{
        HasWindowHandle,
        RawWindowHandle,
    },
    window::{
        Window,
        WindowAttributes,
    },
};

use crate::error::{
    OverlayError,
    Result,
};

/// Creates and configures the overlay window.
pub fn create_window(
    event_loop: &EventLoop<()>,
    title: &str,
    monitor: Option<i32>,
) -> Result<(HWND, Window)> {
    // Create window with default attributes first to query monitors
    let mut window_attributes = WindowAttributes::default()
        .with_title(title.to_owned())
        .with_visible(false)
        .with_decorations(false);

    // If a specific monitor is requested, set window attributes
    if let Some(monitor_idx) = monitor {
        log::info!("Looking for monitor {}", monitor_idx);
        #[allow(deprecated)]
        let temp_window = event_loop.create_window(WindowAttributes::default())?;
        let monitors: Vec<_> = temp_window.available_monitors().collect();
        log::info!("Found {} monitors", monitors.len());

        for (idx, m) in monitors.iter().enumerate() {
            log::info!(
                "Monitor {}: position={:?}, size={:?}, scale_factor={}",
                idx,
                m.position(),
                m.size(),
                m.scale_factor()
            );
        }

        if let Some(monitor) = monitors.get(monitor_idx as usize) {
            let pos = monitor.position();
            let size = monitor.size();
            log::info!(
                "Selected monitor {}, position: {:?}, size: {:?}, scale_factor: {}",
                monitor_idx,
                pos,
                size,
                monitor.scale_factor()
            );

            let scale_factor = monitor.scale_factor();
            let scaled_width = (size.width as f64 / scale_factor).round() as i32;
            let scaled_height = (size.height as f64 / scale_factor).round() as i32;
            let scaled_size = LogicalSize::new(scaled_width, scaled_height);

            log::info!(
                "Setting window size to {} x {}",
                scaled_width,
                scaled_height
            );

            window_attributes = window_attributes
                .with_position(pos)
                .with_inner_size(scaled_size);
        } else {
            log::warn!("Monitor {} not found", monitor_idx);
        }
    }

    #[allow(deprecated)]
    let window = event_loop.create_window(window_attributes)?;

    let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() else {
        panic!()
    };
    let hwnd = HWND(handle.hwnd.get());

    unsafe {
        // Make it transparent
        SetWindowLongA(hwnd, GWL_STYLE, (WS_VISIBLE | WS_CLIPSIBLINGS).0 as i32);
        SetWindowLongPtrA(
            hwnd,
            GWL_EXSTYLE,
            (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_APPWINDOW).0 as isize,
        );

        if !DwmIsCompositionEnabled()?.as_bool() {
            return Err(OverlayError::DwmCompositionDisabled);
        }

        let mut bb: DWM_BLURBEHIND = Default::default();
        bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
        bb.fEnable = BOOL::from(true);
        bb.hRgnBlur = CreateRectRgn(0, 0, 1, 1);
        DwmEnableBlurBehindWindow(hwnd, &bb)?;
        DeleteObject(bb.hRgnBlur);

        ShowWindow(hwnd, SW_SHOW);
    }

    Ok((hwnd, window))
}
