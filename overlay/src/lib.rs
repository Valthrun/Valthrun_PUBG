#![feature(str_from_utf16_endian)]
use std::time::Instant;

use clipboard::ClipboardSupport;
use copypasta::ClipboardContext;
use font::FontAtlasBuilder;
use imgui::{
    Context,
    FontAtlas,
};
use imgui_winit_support::{
    winit::{
        event::{
            Event,
            WindowEvent,
        },
        event_loop::EventLoop,
        window::Window,
    },
    HiDpiMode,
    WinitPlatform,
};
use input::{
    KeyboardInputSystem,
    MouseInputSystem,
};
use obfstr::obfstr;
use opengl::OpenGLRenderBackend;
use vulkan::VulkanRenderBackend;
use window_tracker::ActiveTracker;
use windows::Win32::{
    Foundation::{
        BOOL,
        HWND,
    },
    Graphics::{
        Dwm::{
            DwmEnableBlurBehindWindow,
            DwmIsCompositionEnabled,
            DWM_BB_BLURREGION,
            DWM_BB_ENABLE,
            DWM_BLURBEHIND,
        },
        Gdi::{
            CreateRectRgn,
            DeleteObject,
        },
    },
    UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity,
        SetWindowLongA,
        SetWindowLongPtrA,
        ShowWindow,
        GWL_EXSTYLE,
        GWL_STYLE,
        SW_SHOW,
        WDA_EXCLUDEFROMCAPTURE,
        WDA_NONE,
        WS_CLIPSIBLINGS,
        WS_EX_APPWINDOW,
        WS_EX_LAYERED,
        WS_EX_TRANSPARENT,
        WS_VISIBLE,
    },
};

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
use winit::{
    dpi::LogicalSize,
    raw_window_handle::{
        HasWindowHandle,
        RawWindowHandle,
    },
    window::WindowAttributes,
};

mod render;
pub use render::RenderBackend;

fn create_window(
    event_loop: &EventLoop<()>,
    title: &str,
    monitor: Option<i32>,
) -> Result<(HWND, Window)> {
    // Create window with default attributes first to query monitors
    let mut window_attributes = WindowAttributes::default()
        .with_title(title.to_owned())
        .with_visible(false)
        .with_decorations(false);

    // If a specific monitor is requested, set window attributes
    if let Some(monitor_idx) = monitor {
        log::info!("Looking for monitor {}", monitor_idx);

        // Create a temporary window to query monitors
        #[allow(deprecated)]
        let temp_window = event_loop.create_window(WindowAttributes::default())?;
        let monitors: Vec<_> = temp_window.available_monitors().collect();
        log::info!("Found {} monitors", monitors.len());

        // Log all monitor information
        for (idx, m) in monitors.iter().enumerate() {
            log::info!(
                "Monitor {}: position={:?}, size={:?}, scale_factor={}",
                idx,
                m.position(),
                m.size(),
                m.scale_factor()
            );
        }

        if let Some(monitor) = monitors.get(monitor_idx as usize) {
            let pos = monitor.position();
            let size = monitor.size();
            log::info!(
                "Selected monitor {}, position: {:?}, size: {:?}, scale_factor: {}",
                monitor_idx,
                pos,
                size,
                monitor.scale_factor()
            );

            // Adjust size for scale factor
            let scale_factor = monitor.scale_factor();
            let scaled_width = (size.width as f64 / scale_factor).round() as i32;
            let scaled_height = (size.height as f64 / scale_factor).round() as i32;
            let scaled_size = LogicalSize::new(scaled_width, scaled_height);

            log::info!(
                "Setting window size to {} x {}",
                scaled_width,
                scaled_height
            );

            window_attributes = window_attributes
                .with_position(pos)
                .with_inner_size(scaled_size);
        } else {
            log::warn!("Monitor {} not found", monitor_idx);
        }
    }

    // Create window with final attributes
    #[allow(deprecated)]
    let window = event_loop.create_window(window_attributes)?;

    let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() else {
        panic!()
    };
    let hwnd = HWND(handle.hwnd.get());

    unsafe {
        // Make it transparent
        SetWindowLongA(hwnd, GWL_STYLE, (WS_VISIBLE | WS_CLIPSIBLINGS).0 as i32);
        SetWindowLongPtrA(
            hwnd,
            GWL_EXSTYLE,
            (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_APPWINDOW).0 as isize,
        );

        if !DwmIsCompositionEnabled()?.as_bool() {
            return Err(OverlayError::DwmCompositionDisabled);
        }

        let mut bb: DWM_BLURBEHIND = Default::default();
        bb.dwFlags = DWM_BB_ENABLE | DWM_BB_BLURREGION;
        bb.fEnable = BOOL::from(true);
        bb.hRgnBlur = CreateRectRgn(0, 0, 1, 1);
        DwmEnableBlurBehindWindow(hwnd, &bb)?;
        DeleteObject(bb.hRgnBlur);

        // Show window
        ShowWindow(hwnd, SW_SHOW);
    }

    // Log final window size
    let final_size = window.inner_size();
    let final_pos = window.outer_position().unwrap_or_default();
    log::info!(
        "Final window size: {:?}, position: {:?}",
        final_size,
        final_pos
    );

    Ok((hwnd, window))
}

fn create_imgui_context(_options: &OverlayOptions) -> Result<(WinitPlatform, imgui::Context)> {
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let platform = WinitPlatform::new(&mut imgui);

    match ClipboardContext::new() {
        Ok(backend) => imgui.set_clipboard_backend(ClipboardSupport(backend)),
        Err(error) => log::warn!("Failed to initialize clipboard: {}", error),
    };

    Ok((platform, imgui))
}

pub struct OverlayOptions {
    pub title: String,
    pub register_fonts_callback: Option<Box<dyn Fn(&mut FontAtlas) -> ()>>,
    pub monitor: Option<i32>,
}

pub struct System {
    pub event_loop: EventLoop<()>,

    pub overlay_window: Window,
    pub overlay_hwnd: HWND,

    pub platform: WinitPlatform,

    pub imgui: Context,
    pub imgui_fonts: FontAtlasBuilder,
    pub imgui_register_fonts_callback: Option<Box<dyn Fn(&mut FontAtlas) -> ()>>,

    renderer: Box<dyn RenderBackend>,
}

pub fn init(options: OverlayOptions) -> Result<System> {
    let event_loop = EventLoop::new().unwrap();
    let (overlay_hwnd, overlay_window) =
        create_window(&event_loop, &options.title, options.monitor)?;

    let (mut platform, mut imgui) = create_imgui_context(&options)?;
    platform.attach_window(imgui.io_mut(), &overlay_window, HiDpiMode::Default);

    let mut imgui_fonts = FontAtlasBuilder::new();
    imgui_fonts.register_font(include_bytes!("../resources/Roboto-Regular.ttf"))?;
    imgui_fonts.register_font(include_bytes!("../resources/NotoSansTC-Regular.ttf"))?;
    /* fallback if we do not have the roboto version of the glyph */
    imgui_fonts.register_font(include_bytes!("../resources/unifont-15.1.05.otf"))?;
    imgui_fonts.register_codepoints(1..255);

    let renderer: Box<dyn RenderBackend> =
        if std::env::var("OVERLAY_VULKAN").map_or(false, |v| v == "1") {
            Box::new(VulkanRenderBackend::new(&overlay_window, &mut imgui)?)
        } else {
            Box::new(OpenGLRenderBackend::new(&event_loop, &overlay_window)?)
        };

    Ok(System {
        event_loop,
        overlay_window,
        overlay_hwnd,

        imgui,
        imgui_fonts,
        imgui_register_fonts_callback: options.register_fonts_callback,

        platform,

        renderer,
    })
}

const PERF_RECORDS: usize = 2048;

impl System {
    pub fn main_loop<U, R>(self, mut update: U, mut render: R) -> i32
    where
        U: FnMut(&mut SystemRuntimeController, &Window) -> bool + 'static,
        R: FnMut(&imgui::Ui, &UnicodeTextRenderer) -> bool + 'static,
    {
        let System {
            event_loop,
            overlay_window: window,
            overlay_hwnd,

            imgui,
            imgui_fonts,
            imgui_register_fonts_callback,

            mut platform,

            mut renderer,
            ..
        } = self;

        let mut last_frame = Instant::now();

        let mut runtime_controller = SystemRuntimeController {
            hwnd: overlay_hwnd,

            imgui,
            imgui_fonts,

            active_tracker: ActiveTracker::new(overlay_hwnd),
            key_input_system: KeyboardInputSystem::new(),
            mouse_input_system: MouseInputSystem::new(overlay_hwnd),

            frame_count: 0,
            debug_overlay_shown: false,
        };

        let mut perf = PerfTracker::new(PERF_RECORDS);
        #[allow(deprecated)]
        let _ = event_loop.run(move |event, event_loop| {
            platform.handle_event(runtime_controller.imgui.io_mut(), &window, &event);

            match event {
                // New frame
                Event::NewEvents(_) => {
                    perf.begin();
                    let now = Instant::now();
                    runtime_controller
                        .imgui
                        .io_mut()
                        .update_delta_time(now - last_frame);
                    last_frame = now;
                }

                Event::AboutToWait => {
                    window.request_redraw();
                }

                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    /* Update */
                    {
                        if !runtime_controller.update_state(&window) {
                            event_loop.exit();
                            return;
                        }

                        if !update(&mut runtime_controller, &window) {
                            event_loop.exit();
                            return;
                        }

                        if runtime_controller.imgui_fonts.fetch_reset_flag_updated() {
                            let font_atlas = runtime_controller.imgui.fonts();
                            font_atlas.clear();

                            let (font_sources, _glyph_memory) =
                                runtime_controller.imgui_fonts.build_font_source(18.0);

                            font_atlas.add_font(&font_sources);
                            if let Some(user_callback) = &imgui_register_fonts_callback {
                                user_callback(font_atlas);
                            }

                            renderer.update_fonts_texture(&mut runtime_controller.imgui);
                        }

                        perf.mark("update");
                    }

                    /* Generate frame */
                    let draw_data = {
                        if let Err(error) =
                            platform.prepare_frame(runtime_controller.imgui.io_mut(), &window)
                        {
                            event_loop.exit();
                            log::error!("Platform implementation prepare_frame failed: {}", error);
                            return;
                        }

                        let ui = runtime_controller.imgui.frame();
                        let unicode_text =
                            UnicodeTextRenderer::new(ui, &mut runtime_controller.imgui_fonts);

                        let run = render(ui, &unicode_text);
                        if !run {
                            event_loop.exit();
                            return;
                        }
                        if runtime_controller.debug_overlay_shown {
                            ui.window("Render Debug")
                                .position([200.0, 200.0], imgui::Condition::FirstUseEver)
                                .size([400.0, 400.0], imgui::Condition::FirstUseEver)
                                .build(|| {
                                    ui.text(format!("FPS: {: >4.2}", ui.io().framerate));
                                    ui.same_line_with_pos(100.0);

                                    ui.text(format!(
                                        "Frame Time: {:.2}ms",
                                        ui.io().delta_time * 1000.0
                                    ));
                                    ui.same_line_with_pos(275.0);

                                    ui.text("History length:");
                                    ui.same_line();
                                    let mut history_length = perf.history_length();
                                    ui.set_next_item_width(75.0);
                                    if ui
                                        .input_scalar("##history_length", &mut history_length)
                                        .build()
                                    {
                                        perf.set_history_length(history_length);
                                    }
                                    perf.render(ui, ui.content_region_avail());
                                });
                        }
                        perf.mark("generate frame");

                        platform.prepare_render(ui, &window);
                        runtime_controller.imgui.render()
                    };

                    /* render */
                    renderer.render_frame(&mut perf, &window, draw_data);

                    runtime_controller.frame_rendered();
                    perf.finish("render");
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    event_loop.exit();
                }
                _ => {}
            }
        });
        0
    }

    pub fn switch_monitor(window: &Window, monitor: i32) {
        let monitors: Vec<_> = window.available_monitors().collect();
        if let Some(monitor) = monitors.get(monitor as usize) {
            log::info!(
                "Switching to monitor position: {:?}, size: {:?}",
                monitor.position(),
                monitor.size(),
            );

            // Get monitor info
            let pos = monitor.position();
            let size = monitor.size();

            // Set position first
            window.set_outer_position(pos);

            // Request new size
            match window.request_inner_size(size) {
                Some(new_size) => {
                    if new_size != size {
                        log::warn!("Window size request partially failed - got {:?} instead of requested {:?}", new_size, size);
                    } else {
                        log::info!("Successfully resized window to {:?}", new_size);
                    }
                }
                None => {
                    log::info!("Window size request pending...");
                }
            }
        } else {
            log::error!("Monitor {} not found", monitor);
        }
    }
}

pub struct SystemRuntimeController {
    pub hwnd: HWND,

    pub imgui: imgui::Context,
    pub imgui_fonts: FontAtlasBuilder,

    debug_overlay_shown: bool,

    active_tracker: ActiveTracker,
    mouse_input_system: MouseInputSystem,
    key_input_system: KeyboardInputSystem,

    frame_count: u64,
}

impl SystemRuntimeController {
    fn update_state(&mut self, window: &Window) -> bool {
        self.mouse_input_system.update(window, self.imgui.io_mut());
        self.key_input_system.update(window, self.imgui.io_mut());
        self.active_tracker.update(self.imgui.io());

        true
    }

    fn frame_rendered(&mut self) {
        self.frame_count += 1;
    }

    pub fn toggle_screen_capture_visibility(&self, should_be_visible: bool) {
        unsafe {
            let (target_state, state_name) = if should_be_visible {
                (WDA_NONE, "normal")
            } else {
                (WDA_EXCLUDEFROMCAPTURE, "exclude from capture")
            };

            if !SetWindowDisplayAffinity(self.hwnd, target_state).as_bool() {
                log::warn!(
                    "{} '{}'.",
                    obfstr!("Failed to change overlay display affinity to"),
                    state_name
                );
            }
        }
    }

    pub fn toggle_debug_overlay(&mut self, visible: bool) {
        self.debug_overlay_shown = visible;
    }

    pub fn debug_overlay_shown(&self) -> bool {
        self.debug_overlay_shown
    }
}
