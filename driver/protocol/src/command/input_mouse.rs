use core::ptr;

#[derive(Debug, Clone, Copy)]
pub struct DriverCommandInputMouse {
    pub buffer: *const MouseState,
    pub state_count: usize,
}

impl Default for DriverCommandInputMouse {
    fn default() -> Self {
        Self {
            buffer: ptr::null(),
            state_count: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseState {
    pub buttons: [Option<bool>; 0x05],
    pub hwheel: bool,
    pub wheel: bool,

    pub last_x: i32,
    pub last_y: i32,
}
