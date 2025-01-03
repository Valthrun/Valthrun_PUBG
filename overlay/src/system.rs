use std::{
    env,
    time::Instant,
};

use imgui::{
    Context as ImGuiContext,
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
use windows::Win32::Foundation::HWND;

use crate::{
    // bring in references needed for System
    error::Result,
    font::{
        FontAtlasBuilder,
        UnicodeTextRenderer,
    },
    imgui_context::create_imgui_context,
    input::{
        KeyboardInputSystem,
        MouseInputSystem,
    },
    opengl::OpenGLRenderBackend,
    perf::PerfTracker,
    render::RenderBackend,
    vulkan::VulkanRenderBackend,
    window::create_window,
    window_tracker::ActiveTracker,
};

/// Options for setting up the overlay.  
pub struct OverlayOptions {
    pub title: String,
    pub register_fonts_callback: Option<Box<dyn Fn(&mut FontAtlas) -> ()>>,
    pub monitor: Option<i32>,
}

/// The main overlay System handling event loop, rendering, and input.
pub struct System {
    pub event_loop: EventLoop<()>,
    pub overlay_window: Window,
    pub overlay_hwnd: HWND,

    pub platform: WinitPlatform,

    pub imgui: ImGuiContext,
    pub imgui_fonts: FontAtlasBuilder,
    pub imgui_register_fonts_callback: Option<Box<dyn Fn(&mut FontAtlas) -> ()>>,

    renderer: Box<dyn RenderBackend>,
}

/// Creates and initializes a System instance.
pub fn init(options: OverlayOptions) -> Result<System> {
    let event_loop = EventLoop::new().unwrap();
    let (overlay_hwnd, overlay_window) =
        create_window(&event_loop, &options.title, options.monitor)?;

    let (mut platform, mut imgui) = create_imgui_context(&event_loop)?;
    platform.attach_window(imgui.io_mut(), &overlay_window, HiDpiMode::Default);

    let mut imgui_fonts = FontAtlasBuilder::new();
    imgui_fonts.register_font(include_bytes!("../resources/Roboto-Regular.ttf"))?;
    imgui_fonts.register_font(include_bytes!("../resources/NotoSansTC-Regular.ttf"))?;
    imgui_fonts.register_font(include_bytes!("../resources/unifont-15.1.05.otf"))?;
    imgui_fonts.register_codepoints(1..255);

    let renderer: Box<dyn RenderBackend> = if env::var("OVERLAY_VULKAN").map_or(false, |v| v == "1")
    {
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

impl System {
    /// Runs the main overlay loop, calling user-defined update/render each frame.
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

        let mut perf = PerfTracker::new(2048);

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
                    // -- update --
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

                    // -- generate frame --
                    let draw_data = {
                        if let Err(error) =
                            platform.prepare_frame(runtime_controller.imgui.io_mut(), &window)
                        {
                            event_loop.exit();
                            log::error!("Platform prepare_frame() failed: {}", error);
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

                    // -- render --
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

    /// Switches the overlay window to another monitor index.
    pub fn switch_monitor(window: &Window, monitor: i32) {
        let monitors: Vec<_> = window.available_monitors().collect();
        if let Some(monitor) = monitors.get(monitor as usize) {
            log::info!(
                "Switching to monitor position: {:?}, size: {:?}",
                monitor.position(),
                monitor.size(),
            );

            let pos = monitor.position();
            let size = monitor.size();

            window.set_outer_position(pos);

            match window.request_inner_size(size) {
                Some(new_size) => {
                    if new_size != size {
                        log::warn!(
                            "Window size request partially failed – got {:?} instead of {:?}",
                            new_size,
                            size
                        );
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

/// A minimal subset of data used while the main loop is running.
pub struct SystemRuntimeController {
    pub hwnd: HWND,
    pub imgui: ImGuiContext,
    pub imgui_fonts: FontAtlasBuilder,

    debug_overlay_shown: bool,

    active_tracker: ActiveTracker,
    mouse_input_system: MouseInputSystem,
    key_input_system: KeyboardInputSystem,

    frame_count: u64,
}

impl SystemRuntimeController {
    pub fn toggle_screen_capture_visibility(&self, should_be_visible: bool) {
        use obfstr::obfstr;
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowDisplayAffinity,
            WDA_EXCLUDEFROMCAPTURE,
            WDA_NONE,
        };

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

    pub fn update_state(&mut self, window: &Window) -> bool {
        self.mouse_input_system.update(window, self.imgui.io_mut());
        self.key_input_system.update(window, self.imgui.io_mut());
        self.active_tracker.update(self.imgui.io());
        true
    }

    pub fn frame_rendered(&mut self) {
        self.frame_count += 1;
    }
}
