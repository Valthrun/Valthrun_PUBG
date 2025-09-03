use std::{
    io,
    sync::mpsc,
    time::{
        Duration,
        Instant,
    },
};

use crossterm::{
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode,
        KeyEventKind,
    },
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    style::{
        Color,
        Style,
    },
    text::{
        Line,
        Span,
    },
    widgets::{
        canvas::{
            Canvas,
            Circle,
            Line as CanvasLine,
            Rectangle,
        },
        Block,
        Borders,
        Clear,
        Paragraph,
    },
    Terminal,
};

#[derive(Clone, Debug)]
pub struct RadarPoint {
    pub x: f32,
    pub y: f32,
    pub dz: i32,
    pub health: u32,
}

#[derive(Clone, Debug, Default)]
pub struct RadarFrame {
    pub yaw_deg: f32,
    pub points: Vec<RadarPoint>,
}

pub fn run_tui(
    log_receiver: mpsc::Receiver<Vec<Line<'static>>>,
    radar_receiver: mpsc::Receiver<RadarFrame>,
) -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = app_loop(&mut terminal, log_receiver, radar_receiver).map_err(anyhow::Error::from);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = &res {
        log::error!("Error in TUI: {:?}", err);
    }
    res
}

fn app_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    log_receiver: mpsc::Receiver<Vec<Line<'static>>>,
    radar_receiver: mpsc::Receiver<RadarFrame>,
) -> Result<(), io::Error> {
    let mut persistent_status_lines: Vec<Line<'static>> = Vec::new();
    let mut current_transient_logs: Vec<Line<'static>> = Vec::new();
    const MAX_PERSISTENT_STATUS_LINES: usize = 3; // Show up to 3 persistent status lines
    let mut latest_radar: Option<RadarFrame> = None;
    let mut show_options: bool = false;
    let mut show_height_labels: bool = true;
    let mut show_health_labels: bool = true;
    // Toggle cooldowns to avoid hold-repeat issues across terminals (incl. WSL)
    let mut last_toggle_o: Option<Instant> = None;
    let mut last_toggle_h: Option<Instant> = None;
    let mut last_toggle_z: Option<Instant> = None;
    let mut log_disconnected: bool = false;

    loop {
        match log_receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(new_log_batch) => {
                current_transient_logs.clear();
                for original_line in new_log_batch {
                    let mut is_status = false;
                    let mut clean_status_text = String::new();

                    // The logger prepends a level, e.g., "[ INFO ] [STATUS] text"
                    // The message_span (spans.get(1)) has a leading space from logger's format! call.
                    if let Some(message_span) = original_line.spans.get(1) {
                        let message_content = message_span.content.as_ref();
                        if message_content.trim_start().starts_with("[STATUS]") {
                            is_status = true;
                            clean_status_text = message_content
                                .trim_start()
                                .strip_prefix("[STATUS]")
                                .unwrap_or("")
                                .trim()
                                .to_string();
                        }
                    }

                    if is_status {
                        let status_line = Line::from(clean_status_text);
                        persistent_status_lines.push(status_line);
                        if persistent_status_lines.len() > MAX_PERSISTENT_STATUS_LINES {
                            persistent_status_lines.drain(
                                0..(persistent_status_lines.len() - MAX_PERSISTENT_STATUS_LINES),
                            );
                        }
                    } else {
                        current_transient_logs.push(original_line);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if !log_disconnected {
                    log_disconnected = true;
                    // Add a persistent status line and keep TUI open
                    let status_line = Line::from("[App] log channel closed. Press 'q' to quit.");
                    persistent_status_lines.push(status_line);
                    if persistent_status_lines.len() > MAX_PERSISTENT_STATUS_LINES {
                        persistent_status_lines.drain(
                            0..(persistent_status_lines.len() - MAX_PERSISTENT_STATUS_LINES),
                        );
                    }
                }
            }
        }

        loop {
            match radar_receiver.try_recv() {
                Ok(frame) => latest_radar = Some(frame),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }

        terminal.draw(|f| {
            let top_bottom = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
                .split(f.area());

            let status_area_rect = top_bottom[0];
            let bottom_rect = top_bottom[1];

            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
                .split(bottom_rect);

            let radar_rect = bottom_chunks[0];
            let log_area_rect = bottom_chunks[1];

            let status_paragraph = Paragraph::new(persistent_status_lines.clone())
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status_paragraph, status_area_rect);

            let num_transient_lines = current_transient_logs.len() as u16;
            let transient_panel_height = log_area_rect.height.saturating_sub(2);

            let scroll_offset_y = if num_transient_lines > transient_panel_height {
                num_transient_lines - transient_panel_height
            } else {
                0
            };

            let transient_log_paragraph = Paragraph::new(current_transient_logs.clone())
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .scroll((scroll_offset_y, 0));

            f.render_widget(transient_log_paragraph, log_area_rect);

            // Radar
            let radar_block = Block::default()
                .borders(Borders::ALL)
                .title("Radar (q to quit, o: options)");
            let inner = radar_block.inner(radar_rect);
            f.render_widget(radar_block, radar_rect);

            if let Some(radar) = &latest_radar {
                let yaw_rad: f64 = (radar.yaw_deg as f64).to_radians();

                // World to radar space: rotate so that up is look direction
                let mut radar_points: Vec<(f64, f64, i32)> = Vec::with_capacity(radar.points.len());
                for p in &radar.points {
                    let x = p.x as f64;
                    let y = p.y as f64;
                    let rx = -x * yaw_rad.sin() + y * yaw_rad.cos();
                    let ry = x * yaw_rad.cos() + y * yaw_rad.sin();
                    radar_points.push((rx, ry, p.dz));
                }

                let circles = vec![100.0_f64, 200.0, 300.0, 400.0];
                let x_min: f64 = -400.0;
                let x_max: f64 = 400.0;
                let y_min: f64 = -400.0;
                let y_max: f64 = 400.0;

                let canvas = Canvas::default()
                    .x_bounds([x_min, x_max])
                    .y_bounds([y_min, y_max])
                    .paint(|ctx| {
                        // Background crosshair
                        ctx.draw(&CanvasLine {
                            x1: x_min,
                            y1: 0.0,
                            x2: x_max,
                            y2: 0.0,
                            color: Color::DarkGray,
                        });
                        ctx.draw(&CanvasLine {
                            x1: 0.0,
                            y1: y_min,
                            x2: 0.0,
                            y2: y_max,
                            color: Color::DarkGray,
                        });

                        // Range circles
                        for r in &circles {
                            ctx.draw(&Circle {
                                x: 0.0,
                                y: 0.0,
                                radius: *r,
                                color: Color::Gray,
                            });
                        }

                        // Local player
                        ctx.draw(&Rectangle {
                            x: -3.0,
                            y: -3.0,
                            width: 6.0,
                            height: 6.0,
                            color: Color::Cyan,
                        });

                        if !radar_points.is_empty() {
                            let desired_cells_radius = 0.25;
                            let meters_per_cell_x = (x_max - x_min) / inner.width.max(1) as f64;
                            let r = desired_cells_radius * meters_per_cell_x;
                            let step = meters_per_cell_x * 0.6;
                            for (x, y, _) in &radar_points {
                                let mut rr = r;
                                for _ in 0..3 {
                                    if rr <= 0.0 {
                                        break;
                                    }
                                    ctx.draw(&Circle {
                                        x: *x,
                                        y: *y,
                                        radius: rr,
                                        color: Color::Red,
                                    });
                                    rr -= step;
                                }
                            }
                        }
                    });

                f.render_widget(canvas.block(Block::default()), inner);

                if show_height_labels || show_health_labels {
                    // Skip if inner area has zero size to avoid OOB writes
                    if inner.width == 0 || inner.height == 0 { /* nothing to draw */
                    } else {
                        for p in &radar.points {
                            // Rotate coordinates same as points to align label near marker
                            let x = p.x as f64;
                            let y = p.y as f64;
                            let rx = -x * yaw_rad.sin() + y * yaw_rad.cos();
                            let ry = x * yaw_rad.cos() + y * yaw_rad.sin();

                            let mut parts: Vec<String> = Vec::new();
                            if show_health_labels {
                                parts.push(format!("{}", p.health));
                            }
                            if show_height_labels {
                                parts.push(format!("{:+}m", p.dz));
                            }
                            if parts.is_empty() {
                                continue;
                            }
                            let label = parts.join(" ");

                            // Map to inner rect
                            let nx = ((rx - x_min) / (x_max - x_min)).max(0.0).min(1.0);
                            let ny = ((ry - y_min) / (y_max - y_min)).max(0.0).min(1.0);
                            let gx_center =
                                inner.x.saturating_add((nx * inner.width as f64) as u16);
                            let gy_px = inner
                                .y
                                .saturating_add(((1.0 - ny) * inner.height as f64) as u16);

                            // Place label one row below the dot and horizontally centered
                            let label_width = (label.len() as u16).min(inner.width);
                            if label_width == 0 {
                                continue;
                            }

                            // Compute clamped x so [x, x+width) fits inside inner
                            let inner_x_end = inner.x.saturating_add(inner.width.saturating_sub(1));
                            let gx_unclamped = gx_center.saturating_sub(label_width / 2);
                            let gx_clamped_left = gx_unclamped.max(inner.x);
                            let max_start =
                                inner_x_end.saturating_sub(label_width.saturating_sub(1));
                            let gx = gx_clamped_left.min(max_start);

                            // Compute clamped y so it stays within inner (one row below marker)
                            let mut gy = gy_px.saturating_add(1);
                            let inner_y_end =
                                inner.y.saturating_add(inner.height.saturating_sub(1));
                            if gy > inner_y_end {
                                gy = inner_y_end;
                            }
                            if gy < inner.y {
                                continue;
                            }

                            let area = ratatui::layout::Rect {
                                x: gx,
                                y: gy,
                                width: label_width,
                                height: 1,
                            };
                            let style = Style::default().fg(Color::LightRed);
                            let paragraph = Paragraph::new(Span::styled(label, style));
                            f.render_widget(paragraph, area);
                        }
                    }
                }
            } else {
                let paragraph = Paragraph::new("Waiting for radar data...")
                    .block(Block::default())
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(paragraph, inner);
            }

            // Options overlay
            if show_options {
                // Clamp panel size to bottom area to avoid OOB
                let avail_w = bottom_rect.width;
                let avail_h = bottom_rect.height;
                if avail_w > 2 && avail_h > 2 {
                    let w = 32u16.min(avail_w.saturating_sub(2));
                    let h = 6u16.min(avail_h.saturating_sub(2));
                    let x = bottom_rect.x + avail_w.saturating_sub(w + 1);
                    let y = bottom_rect.y + 1;
                    let area = ratatui::layout::Rect {
                        x,
                        y,
                        width: w,
                        height: h,
                    };
                    f.render_widget(Clear, area);
                    let lines = vec![
                        Line::from("Options"),
                        Line::from(format!(
                            "[h] Show health: {}",
                            if show_health_labels { "On" } else { "Off" }
                        )),
                        Line::from(format!(
                            "[z] Show height: {}",
                            if show_height_labels { "On" } else { "Off" }
                        )),
                        Line::from("[o] Close"),
                    ];
                    let panel = Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL).title("Options"));
                    f.render_widget(panel, area);
                }
            }
        })?;

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                let now = Instant::now();
                let cooldown = Duration::from_millis(150);
                match key.code {
                    KeyCode::Char('o') if key.kind == KeyEventKind::Press => {
                        if last_toggle_o.map_or(true, |t| now.duration_since(t) > cooldown) {
                            last_toggle_o = Some(now);
                            show_options = !show_options;
                        }
                    }
                    KeyCode::Char('h') if key.kind == KeyEventKind::Press => {
                        if last_toggle_h.map_or(true, |t| now.duration_since(t) > cooldown) {
                            last_toggle_h = Some(now);
                            show_health_labels = !show_health_labels;
                        }
                    }
                    KeyCode::Char('z') if key.kind == KeyEventKind::Press => {
                        if last_toggle_z.map_or(true, |t| now.duration_since(t) > cooldown) {
                            last_toggle_z = Some(now);
                            show_height_labels = !show_height_labels;
                        }
                    }
                    KeyCode::Char('q') if key.kind == KeyEventKind::Press => break,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
