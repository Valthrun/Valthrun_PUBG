use imgui::{
    Io,
    Key,
    MouseButton,
};
use imgui_winit_support::winit::window::Window;
use windows::Win32::{
    Foundation::{
        HWND,
        POINT,
    },
    Graphics::Gdi::ScreenToClient,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState,
            VIRTUAL_KEY,
            VK_CONTROL,
            VK_LBUTTON,
            VK_LCONTROL,
            VK_LMENU,
            VK_LSHIFT,
            VK_LWIN,
            VK_MBUTTON,
            VK_MENU,
            VK_RBUTTON,
            VK_RMENU,
            VK_RSHIFT,
            VK_RWIN,
            VK_XBUTTON1,
            VK_XBUTTON2,
        },
        WindowsAndMessaging::GetCursorPos,
    },
};

const VK_KEY_MAX: usize = 256;

/// Trait for reading keyboard input state
pub trait KeyboardInput {
    /// Check if a key is currently held down
    fn is_key_down(&self, key: Key) -> bool;

    /// Check if a key was just pressed
    /// If repeating is true, will trigger multiple times while held
    fn is_key_pressed(&self, key: Key, repeating: bool) -> bool;
}

/// Trait for reading mouse input state
pub trait MouseInput {
    /// Get the current mouse position in screen coordinates
    fn mouse_position(&self) -> [f32; 2];

    /// Check if a mouse button is currently held down
    fn is_button_down(&self, button: MouseButton) -> bool;

    /// Check if a mouse button was just pressed
    fn is_button_pressed(&self, button: MouseButton) -> bool;
}

/// Combined input interface for both keyboard and mouse
pub trait InputSystem: KeyboardInput + MouseInput {
    /// Update the input state
    fn update(&mut self, window: &Window, io: &mut Io);
}

#[derive(Debug, Default)]
pub struct MouseInputSystem {
    hwnd: HWND,
    position: [f32; 2],
}

impl MouseInputSystem {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            position: [0.0, 0.0],
        }
    }

    pub fn update(&mut self, window: &Window, io: &mut Io) {
        let mut point: POINT = Default::default();
        unsafe {
            GetCursorPos(&mut point);
            ScreenToClient(self.hwnd, &mut point);
        };

        self.position = [
            (point.x as f64 / window.scale_factor()) as f32,
            (point.y as f64 / window.scale_factor()) as f32,
        ];
        io.add_mouse_pos_event(self.position);
    }
}

impl MouseInput for MouseInputSystem {
    fn mouse_position(&self) -> [f32; 2] {
        self.position
    }

    fn is_button_down(&self, button: MouseButton) -> bool {
        // Use the ImGui IO state since we're already tracking it there
        unsafe {
            let key_state = match button {
                MouseButton::Left => GetAsyncKeyState(VK_LBUTTON.0 as i32),
                MouseButton::Right => GetAsyncKeyState(VK_RBUTTON.0 as i32),
                MouseButton::Middle => GetAsyncKeyState(VK_MBUTTON.0 as i32),
                MouseButton::Extra1 => GetAsyncKeyState(VK_XBUTTON1.0 as i32),
                MouseButton::Extra2 => GetAsyncKeyState(VK_XBUTTON2.0 as i32),
            } as u16;
            (key_state & 0x8000) > 0
        }
    }

    fn is_button_pressed(&self, button: MouseButton) -> bool {
        // Use the ImGui IO state since we're already tracking it there
        unsafe {
            let key_state = match button {
                MouseButton::Left => GetAsyncKeyState(VK_LBUTTON.0 as i32),
                MouseButton::Right => GetAsyncKeyState(VK_RBUTTON.0 as i32),
                MouseButton::Middle => GetAsyncKeyState(VK_MBUTTON.0 as i32),
                MouseButton::Extra1 => GetAsyncKeyState(VK_XBUTTON1.0 as i32),
                MouseButton::Extra2 => GetAsyncKeyState(VK_XBUTTON2.0 as i32),
            } as u16;
            (key_state & 0x1) > 0
        }
    }
}

/// Simple input system using the global mouse / keyboard state.
/// This does not require the need to process window messages or the imgui overlay to be active.
#[derive(Debug, Default)]
pub struct KeyboardInputSystem {
    key_states: Vec<bool>,
}

impl KeyboardInputSystem {
    pub fn new() -> Self {
        Self {
            key_states: vec![false; VK_KEY_MAX],
        }
    }

    pub fn update(&mut self, _window: &Window, io: &mut Io) {
        for vkey in 0..VK_KEY_MAX {
            let key_state = unsafe { GetAsyncKeyState(vkey as i32) as u16 };
            let pressed = (key_state & 0x8000) > 0;
            if self.key_states[vkey] == pressed {
                continue;
            }

            self.key_states[vkey] = pressed;
            let vkey = VIRTUAL_KEY(vkey as u16);

            handle_key_modifier(io, vkey, pressed);
            let mouse_button = match vkey {
                VK_LBUTTON => Some(MouseButton::Left),
                VK_RBUTTON => Some(MouseButton::Right),
                VK_MBUTTON => Some(MouseButton::Middle),
                VK_XBUTTON1 => Some(MouseButton::Extra1),
                VK_XBUTTON2 => Some(MouseButton::Extra2),
                _ => None,
            };

            if let Some(button) = mouse_button {
                io.add_mouse_button_event(button, pressed);
            } else if let Some(key) = to_imgui_key(vkey) {
                io.add_key_event(key, pressed);
            } else {
                log::trace!("Missing ImGui key for {:?}", vkey);
            }
        }
    }
}

impl KeyboardInput for KeyboardInputSystem {
    fn is_key_down(&self, key: Key) -> bool {
        if let Some(vkey) = from_imgui_key(key) {
            self.key_states[vkey.0 as usize]
        } else {
            false
        }
    }

    fn is_key_pressed(&self, key: Key, _repeating: bool) -> bool {
        if let Some(vkey) = from_imgui_key(key) {
            unsafe {
                let key_state = GetAsyncKeyState(vkey.0 as i32) as u16;
                (key_state & 0x1) > 0
            }
        } else {
            false
        }
    }
}

/// Combined input system that handles both keyboard and mouse input
#[derive(Debug, Default)]
pub struct CombinedInputSystem {
    keyboard: KeyboardInputSystem,
    mouse: MouseInputSystem,
}

impl CombinedInputSystem {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            keyboard: KeyboardInputSystem::new(),
            mouse: MouseInputSystem::new(hwnd),
        }
    }
}

impl KeyboardInput for CombinedInputSystem {
    fn is_key_down(&self, key: Key) -> bool {
        self.keyboard.is_key_down(key)
    }

    fn is_key_pressed(&self, key: Key, repeating: bool) -> bool {
        self.keyboard.is_key_pressed(key, repeating)
    }
}

impl MouseInput for CombinedInputSystem {
    fn mouse_position(&self) -> [f32; 2] {
        self.mouse.mouse_position()
    }

    fn is_button_down(&self, button: MouseButton) -> bool {
        self.mouse.is_button_down(button)
    }

    fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse.is_button_pressed(button)
    }
}

impl InputSystem for CombinedInputSystem {
    fn update(&mut self, window: &Window, io: &mut Io) {
        self.keyboard.update(window, io);
        self.mouse.update(window, io);
    }
}

fn to_imgui_key(keycode: VIRTUAL_KEY) -> Option<Key> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    match keycode {
        VK_TAB => Some(Key::Tab),
        VK_LEFT => Some(Key::LeftArrow),
        VK_RIGHT => Some(Key::RightArrow),
        VK_SHIFT => Some(Key::LeftShift),
        VK_MENU => Some(Key::LeftAlt),
        VK_UP => Some(Key::UpArrow),
        VK_DOWN => Some(Key::DownArrow),
        VK_PRIOR => Some(Key::PageUp),
        VK_NEXT => Some(Key::PageDown),
        VK_HOME => Some(Key::Home),
        VK_END => Some(Key::End),
        VK_INSERT => Some(Key::Insert),
        VK_DELETE => Some(Key::Delete),
        VK_BACK => Some(Key::Backspace),
        VK_SPACE => Some(Key::Space),
        VK_RETURN => Some(Key::Enter),
        VK_ESCAPE => Some(Key::Escape),
        VK_OEM_7 => Some(Key::Apostrophe),
        VK_OEM_COMMA => Some(Key::Comma),
        VK_OEM_MINUS => Some(Key::Minus),
        VK_OEM_PERIOD => Some(Key::Period),
        VK_OEM_2 => Some(Key::Slash),
        VK_OEM_1 => Some(Key::Semicolon),
        VK_OEM_PLUS => Some(Key::Equal),
        VK_OEM_4 => Some(Key::LeftBracket),
        VK_OEM_5 => Some(Key::Backslash),
        VK_OEM_6 => Some(Key::RightBracket),
        VK_OEM_3 => Some(Key::GraveAccent),
        VK_CAPITAL => Some(Key::CapsLock),
        VK_SCROLL => Some(Key::ScrollLock),
        VK_NUMLOCK => Some(Key::NumLock),
        VK_SNAPSHOT => Some(Key::PrintScreen),
        VK_PAUSE => Some(Key::Pause),
        VK_NUMPAD0 => Some(Key::Keypad0),
        VK_NUMPAD1 => Some(Key::Keypad1),
        VK_NUMPAD2 => Some(Key::Keypad2),
        VK_NUMPAD3 => Some(Key::Keypad3),
        VK_NUMPAD4 => Some(Key::Keypad4),
        VK_NUMPAD5 => Some(Key::Keypad5),
        VK_NUMPAD6 => Some(Key::Keypad6),
        VK_NUMPAD7 => Some(Key::Keypad7),
        VK_NUMPAD8 => Some(Key::Keypad8),
        VK_NUMPAD9 => Some(Key::Keypad9),
        VK_DECIMAL => Some(Key::KeypadDecimal),
        VK_DIVIDE => Some(Key::KeypadDivide),
        VK_MULTIPLY => Some(Key::KeypadMultiply),
        VK_SUBTRACT => Some(Key::KeypadSubtract),
        VK_ADD => Some(Key::KeypadAdd),
        VK_LSHIFT => Some(Key::LeftShift),
        VK_LCONTROL | VK_CONTROL => Some(Key::LeftCtrl),
        VK_RCONTROL => Some(Key::RightCtrl),
        VK_LMENU => Some(Key::LeftAlt),
        VK_LWIN => Some(Key::LeftSuper),
        VK_RSHIFT => Some(Key::RightShift),
        VK_RMENU => Some(Key::RightAlt),
        VK_RWIN => Some(Key::RightSuper),
        VK_APPS => Some(Key::Menu),
        VK_0 => Some(Key::Alpha0),
        VK_1 => Some(Key::Alpha1),
        VK_2 => Some(Key::Alpha2),
        VK_3 => Some(Key::Alpha3),
        VK_4 => Some(Key::Alpha4),
        VK_5 => Some(Key::Alpha5),
        VK_6 => Some(Key::Alpha6),
        VK_7 => Some(Key::Alpha7),
        VK_8 => Some(Key::Alpha8),
        VK_9 => Some(Key::Alpha9),
        VK_A => Some(Key::A),
        VK_B => Some(Key::B),
        VK_C => Some(Key::C),
        VK_D => Some(Key::D),
        VK_E => Some(Key::E),
        VK_F => Some(Key::F),
        VK_G => Some(Key::G),
        VK_H => Some(Key::H),
        VK_I => Some(Key::I),
        VK_J => Some(Key::J),
        VK_K => Some(Key::K),
        VK_L => Some(Key::L),
        VK_M => Some(Key::M),
        VK_N => Some(Key::N),
        VK_O => Some(Key::O),
        VK_P => Some(Key::P),
        VK_Q => Some(Key::Q),
        VK_R => Some(Key::R),
        VK_S => Some(Key::S),
        VK_T => Some(Key::T),
        VK_U => Some(Key::U),
        VK_V => Some(Key::V),
        VK_W => Some(Key::W),
        VK_X => Some(Key::X),
        VK_Y => Some(Key::Y),
        VK_Z => Some(Key::Z),
        VK_F1 => Some(Key::F1),
        VK_F2 => Some(Key::F2),
        VK_F3 => Some(Key::F3),
        VK_F4 => Some(Key::F4),
        VK_F5 => Some(Key::F5),
        VK_F6 => Some(Key::F6),
        VK_F7 => Some(Key::F7),
        VK_F8 => Some(Key::F8),
        VK_F9 => Some(Key::F9),
        VK_F10 => Some(Key::F10),
        VK_F11 => Some(Key::F11),
        VK_F12 => Some(Key::F12),
        _ => None,
    }
}

fn from_imgui_key(key: Key) -> Option<VIRTUAL_KEY> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    match key {
        Key::Tab => Some(VK_TAB),
        Key::LeftArrow => Some(VK_LEFT),
        Key::RightArrow => Some(VK_RIGHT),
        Key::LeftShift => Some(VK_LSHIFT),
        Key::LeftAlt => Some(VK_LMENU),
        Key::UpArrow => Some(VK_UP),
        Key::DownArrow => Some(VK_DOWN),
        Key::PageUp => Some(VK_PRIOR),
        Key::PageDown => Some(VK_NEXT),
        Key::Home => Some(VK_HOME),
        Key::End => Some(VK_END),
        Key::Insert => Some(VK_INSERT),
        Key::Delete => Some(VK_DELETE),
        Key::Backspace => Some(VK_BACK),
        Key::Space => Some(VK_SPACE),
        Key::Enter => Some(VK_RETURN),
        Key::Escape => Some(VK_ESCAPE),
        _ => None,
    }
}

fn handle_key_modifier(io: &mut Io, key: VIRTUAL_KEY, down: bool) {
    if key == VK_LSHIFT || key == VK_RSHIFT {
        io.add_key_event(Key::ModShift, down);
    } else if key == VK_LCONTROL || key == VK_CONTROL {
        io.add_key_event(Key::ModCtrl, down);
    } else if key == VK_MENU || key == VK_LMENU || key == VK_RMENU {
        io.add_key_event(Key::ModAlt, down);
    } else if key == VK_LWIN || key == VK_RWIN {
        io.add_key_event(Key::ModSuper, down);
    }
}
