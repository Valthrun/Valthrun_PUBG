#![feature(str_from_utf16_endian)]

mod clipboard;
mod error;
pub use error::*;
mod input;
mod window_tracker;

mod opengl;
mod vulkan;

mod perf;
pub use perf::PerfTracker;

mod font;
mod util;

pub use font::UnicodeTextRenderer;
pub use util::show_error_message;

mod render;
pub use render::RenderBackend;

mod imgui_context;
mod system;
mod window;

pub use system::{
    init,
    OverlayOptions,
    System,
    SystemRuntimeController,
};
