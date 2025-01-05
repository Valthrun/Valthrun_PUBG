use std::{
    io::{
        self,
        Write,
    },
    str::FromStr,
    sync::Mutex,
};

use log::{
    Level,
    LevelFilter,
    Log,
    Metadata,
    Record,
    SetLoggerError,
};
use once_cell::sync::Lazy;

static FRAME_LOG_BUFFER: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static CONSOLE_HEIGHT: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(30)); // default height
static FIRST_FLUSH: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

pub struct FrameLogger;

impl Log for FrameLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_color = match record.level() {
                Level::Error => "\x1B[31m", // Red
                Level::Warn => "\x1B[33m",  // Yellow
                Level::Info => "\x1B[32m",  // Green
                Level::Debug => "\x1B[36m", // Cyan
                Level::Trace => "\x1B[90m", // Bright black (gray)
            };
            let reset = "\x1B[0m";
            let log_entry = format!(
                "{}[{}]{} {}",
                level_color,
                record.level(),
                reset,
                record.args()
            );
            FRAME_LOG_BUFFER.lock().unwrap().push(log_entry);
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    // Clear screen and scroll buffer, hide cursor
    print!("\x1B[2J\x1B[3J\x1B[H\x1B[?25l");
    io::stdout().flush().unwrap();

    // Read from RUST_LOG environment variable and set max level
    let max_level = std::env::var("RUST_LOG")
        .map(|level| LevelFilter::from_str(&level).unwrap_or(LevelFilter::Info))
        .unwrap_or(LevelFilter::Info);

    // Set our custom logger
    log::set_logger(&FrameLogger).map(|()| log::set_max_level(max_level))
}

pub fn flush_frame_logs() {
    let mut frame_buffer = FRAME_LOG_BUFFER.lock().unwrap();
    let height = *CONSOLE_HEIGHT.lock().unwrap();
    let mut is_first_flush = FIRST_FLUSH.lock().unwrap();

    if *is_first_flush {
        // On first flush, print everything from home position
        print!("\x1B[H");
        for (i, log) in frame_buffer.iter().enumerate() {
            print!("\x1B[{};0H\x1B[K{}", i + 1, log);
            io::stdout().flush().unwrap();
        }
        *is_first_flush = false;
    } else {
        // On subsequent flushes, reserve first 3 lines
        let available_height = height.saturating_sub(3);
        let start = if frame_buffer.len() > available_height {
            frame_buffer.len() - available_height
        } else {
            0
        };

        print!("\x1B[4;0H");
        io::stdout().flush().unwrap();

        for (i, log) in frame_buffer.iter().skip(start).enumerate() {
            print!("\x1B[{};0H\x1B[K{}", i + 4, log);
            io::stdout().flush().unwrap();
        }

        for i in frame_buffer.len()..available_height {
            print!("\x1B[{};0H\x1B[K", i + 4);
            io::stdout().flush().unwrap();
        }
    }

    frame_buffer.clear();
}
