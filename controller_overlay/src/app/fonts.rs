use std::{
    cell::RefCell,
    sync::Arc,
};

use imgui::{FontConfig, FontId, FontSource};

/// A reference to a font that can be safely shared and modified across threads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FontReference {
    inner: Arc<RefCell<Option<FontId>>>,
}

impl FontReference {
    /// Gets the current font ID if one is set.
    pub fn font_id(&self) -> Option<FontId> {
        self.inner.borrow().clone()
    }

    /// Sets the font ID for this reference.
    pub fn set_id(&self, font_id: FontId) {
        *self.inner.borrow_mut() = Some(font_id);
    }
}

/// Collection of fonts used by the application.
#[derive(Clone, Default)]
pub struct AppFonts {
    pub valthrun: FontReference,
}

impl AppFonts {
    /// Creates a new font collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a font configuration callback for the ImGui atlas.
    pub fn create_font_config_callback(app_fonts: AppFonts) -> Box<dyn Fn(&mut imgui::FontAtlas)> {
        Box::new(move |atlas| {
            let font_size = 18.0;
            let valthrun_font = atlas.add_font(&[FontSource::TtfData {
                data: include_bytes!("../../resources/Valthrun-Regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.5,
                    oversample_h: 4,
                    oversample_v: 4,
                    ..FontConfig::default()
                }),
            }]);

            app_fonts.valthrun.set_id(valthrun_font);
        })
    }
} 