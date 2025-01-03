use imgui::DrawData;
use winit::window::Window;

use crate::PerfTracker;

pub mod backend;
pub use backend::RenderBackendType;

/// Trait defining the interface for rendering backends
pub trait RenderBackend {
    /// Updates the font texture in the rendering backend
    fn update_fonts_texture(&mut self, imgui: &mut imgui::Context);

    /// Renders a frame using the provided draw data
    fn render_frame(&mut self, perf: &mut PerfTracker, window: &Window, draw_data: &DrawData);

    /// Returns the type of the rendering backend
    fn backend_type(&self) -> RenderBackendType;

    /// Initializes any backend-specific resources
    fn initialize(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Cleans up any backend-specific resources
    fn cleanup(&mut self) {}
}
