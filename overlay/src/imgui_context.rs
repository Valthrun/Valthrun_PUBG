use copypasta::ClipboardContext;
use imgui::Context;
use imgui_winit_support::{
    winit::event_loop::EventLoop,
    WinitPlatform,
};
use log::warn;

use crate::{
    clipboard::ClipboardSupport,
    error::Result,
};

/// Creates and configures the imgui Context.
pub fn create_imgui_context(_event_loop: &EventLoop<()>) -> Result<(WinitPlatform, Context)> {
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let platform = WinitPlatform::new(&mut imgui);

    match ClipboardContext::new() {
        Ok(backend) => imgui.set_clipboard_backend(ClipboardSupport(backend)),
        Err(error) => {
            warn!("Failed to initialize clipboard: {}", error);
        }
    };

    Ok((platform, imgui))
}

/// A helper type for passing extra configuration to the ImGui context if needed:
#[derive(Default)]
pub struct ImGuiContextOptions {}
