use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::SetActiveWindow,
        WindowsAndMessaging::{
            GetWindowLongPtrA,
            SetWindowLongPtrA,
            GWL_EXSTYLE,
            WS_EX_NOACTIVATE,
            WS_EX_TRANSPARENT,
        },
    },
};

/// Toggles the overlay noactive and transparent state
/// according to whenever ImGui wants mouse/cursor grab.
pub struct ActiveTracker {
    hwnd: HWND,
    currently_active: bool,
}

impl ActiveTracker {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            currently_active: true,
        }
    }

    pub fn update(&mut self, io: &imgui::Io) {
        let window_active = io.want_capture_mouse | io.want_capture_keyboard;
        if window_active == self.currently_active {
            return;
        }

        self.currently_active = window_active;
        unsafe {
            let mut style = GetWindowLongPtrA(self.hwnd, GWL_EXSTYLE);
            if window_active {
                style &= !((WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize);
            } else {
                style |= (WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize;
            }

            log::trace!("Set UI active: {window_active}");
            SetWindowLongPtrA(self.hwnd, GWL_EXSTYLE, style);
            if window_active {
                SetActiveWindow(self.hwnd);
            }
        }
    }
}
