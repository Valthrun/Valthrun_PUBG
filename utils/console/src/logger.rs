use std::{
    fs::OpenOptions,
    io::Write,
    sync::Mutex,
};

use log::{
    Level,
    Log,
    Metadata,
    Record,
    SetLoggerError,
};
use once_cell::sync::Lazy;
use ratatui::{
    style::{
        Color,
        Style,
    },
    text::{
        Line,
        Span,
    },
};

static FRAME_LOG_BUFFER: Lazy<Mutex<Vec<Line>>> = Lazy::new(|| Mutex::new(Vec::new()));
static LOG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| Mutex::new(None));
const MAX_LOG_LINES: usize = 1000;

pub struct RatatuiLogger;

impl Log for RatatuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_color = match record.level() {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Cyan,
                Level::Trace => Color::Gray,
            };

            let level_span = Span::styled(
                format!("[ {} ]", record.level()),
                Style::default().fg(level_color),
            );
            let message_span = Span::raw(format!(" {}", record.args()));

            let line = Line::from(vec![level_span, message_span]);

            // Write to TUI buffer
            let mut buffer = FRAME_LOG_BUFFER.lock().unwrap();
            buffer.push(line);

            if buffer.len() > MAX_LOG_LINES {
                let to_remove = buffer.len() - MAX_LOG_LINES;
                buffer.drain(0..to_remove);
            }

            // Write to file if enabled
            if let Ok(mut file_opt) = LOG_FILE.lock() {
                if let Some(ref mut file) = *file_opt {
                    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    let file_line =
                        format!("{} [ {} ] {}\n", timestamp, record.level(), record.args());
                    let _ = file.write_all(file_line.as_bytes());
                    let _ = file.flush(); // Ensure immediate write
                }
            }
        }
    }

    fn flush(&self) {
        // Flush file logger
        if let Ok(mut file_opt) = LOG_FILE.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = file.flush();
            }
        }
    }
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&RatatuiLogger).map(|()| log::set_max_level(log::LevelFilter::Info))
}

pub fn enable_file_logging(log_file_path: &str) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;

    let mut log_file = LOG_FILE.lock().unwrap();
    *log_file = Some(file);

    Ok(())
}

pub fn disable_file_logging() {
    let mut log_file = LOG_FILE.lock().unwrap();
    *log_file = None;
}

/// Retrieves all currently buffered log lines and then clears the buffer.
pub fn get_and_clear_log_lines() -> Vec<Line<'static>> {
    let mut buffer = FRAME_LOG_BUFFER.lock().unwrap();
    let lines: Vec<Line> = buffer.drain(..).collect();
    lines
}
