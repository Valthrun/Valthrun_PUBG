mod console_io;
pub use console_io::*;

mod logger;
pub use logger::{
    get_and_clear_log_lines,
    init_logger,
    RatatuiLogger,
};

mod tui;
pub use tui::run_tui;
