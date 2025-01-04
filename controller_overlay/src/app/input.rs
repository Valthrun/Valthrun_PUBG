use std::ops::Deref;
use imgui::{Ui, MouseButton, Key};
use overlay::input::{KeyboardInput, MouseInput};

pub struct UiInput<'a>(pub &'a Ui);

impl<'a> Deref for UiInput<'a> {
    type Target = Ui;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> KeyboardInput for UiInput<'a> {
    fn is_key_down(&self, key: Key) -> bool {
        Ui::is_key_down(self.0, key)
    }

    fn is_key_pressed(&self, key: Key, repeating: bool) -> bool {
        if repeating {
            Ui::is_key_pressed(self.0, key)
        } else {
            Ui::is_key_pressed_no_repeat(self.0, key)
        }
    }
}

impl<'a> MouseInput for UiInput<'a> {
    fn mouse_position(&self) -> [f32; 2] {
        self.0.io().mouse_pos
    }

    fn is_button_down(&self, button: MouseButton) -> bool {
        Ui::is_mouse_down(self.0, button)
    }

    fn is_button_pressed(&self, button: MouseButton) -> bool {
        Ui::is_mouse_clicked(self.0, button)
    }
}
