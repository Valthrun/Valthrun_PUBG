use std::sync::Mutex;

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

            let mut buffer = FRAME_LOG_BUFFER.lock().unwrap();
            buffer.push(line);

            if buffer.len() > MAX_LOG_LINES {
                let to_remove = buffer.len() - MAX_LOG_LINES;
                buffer.drain(0..to_remove);
            }
        }
    }

    fn flush(&self) {
        // Flushing is handled by the TUI drawing loop, so this can be a no-op.
    }
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&RatatuiLogger).map(|()| log::set_max_level(log::LevelFilter::Info))
}

/// Retrieves all currently buffered log lines and then clears the buffer.
pub fn get_and_clear_log_lines() -> Vec<Line<'static>> {
    let mut buffer = FRAME_LOG_BUFFER.lock().unwrap();
    let lines: Vec<Line> = buffer.drain(..).collect();
    lines
}
