use std::{
    io,
    sync::mpsc,
    time::Duration,
};

use crossterm::{
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode,
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
    text::Line,
    widgets::{
        Block,
        Borders,
        Paragraph,
    },
    Terminal,
};

pub fn run_tui(log_receiver: mpsc::Receiver<Vec<Line<'static>>>) -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = app_loop(&mut terminal, log_receiver).map_err(anyhow::Error::from);

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
) -> Result<(), io::Error> {
    let mut persistent_status_lines: Vec<Line<'static>> = Vec::new();
    let mut current_transient_logs: Vec<Line<'static>> = Vec::new();
    const MAX_PERSISTENT_STATUS_LINES: usize = 3; // Show up to 3 persistent status lines

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
                log::info!("TUI: Log channel disconnected, exiting loop.");
                break;
            }
        }

        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                // Allocate 5 lines for status area (3 text lines + 2 for borders)
                .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
                .split(f.size());

            let status_area_rect = main_chunks[0];
            let log_area_rect = main_chunks[1];

            let status_paragraph = Paragraph::new(persistent_status_lines.clone())
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status_paragraph, status_area_rect);

            let num_transient_lines = current_transient_logs.len() as u16;
            let transient_panel_height = log_area_rect.height.saturating_sub(2); // -2 for Block borders

            let scroll_offset_y = if num_transient_lines > transient_panel_height {
                num_transient_lines - transient_panel_height
            } else {
                0
            };

            let transient_log_paragraph = Paragraph::new(current_transient_logs.clone())
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .scroll((scroll_offset_y, 0));

            f.render_widget(transient_log_paragraph, log_area_rect);
        })?;

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
    Ok(())
}
