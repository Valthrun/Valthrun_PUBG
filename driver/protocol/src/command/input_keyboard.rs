use core::ptr;

#[derive(Debug, Clone, Copy)]
pub struct DriverCommandInputKeyboard {
    pub buffer: *const KeyboardState,
    pub state_count: usize,
}

impl Default for DriverCommandInputKeyboard {
    fn default() -> Self {
        Self {
            buffer: ptr::null(),
            state_count: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KeyboardState {
    pub scane_code: u16,
    pub down: bool,
}
