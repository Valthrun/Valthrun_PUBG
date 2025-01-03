/// Enum representing different rendering backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendType {
    OpenGL,
    Vulkan,
}

impl RenderBackendType {
    pub fn name(&self) -> &'static str {
        match self {
            RenderBackendType::OpenGL => "OpenGL",
            RenderBackendType::Vulkan => "Vulkan",
        }
    }
}
