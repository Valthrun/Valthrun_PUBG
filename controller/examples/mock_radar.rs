use std::{
    sync::mpsc,
    thread,
    time::Duration,
};

use ratatui::text::Line;
use utils_console::{
    self,
    RadarFrame,
    RadarPoint,
};

fn main() {
    // Init logger and channels
    utils_console::init_logger().expect("logger");
    let (log_tx, log_rx) = mpsc::channel::<Vec<Line<'static>>>();
    let (radar_tx, radar_rx) = mpsc::channel::<RadarFrame>();

    // Mock data producer
    thread::spawn(move || {
        let mut t: f32 = 0.0;
        loop {
            // Log some lines
            log::info!("[STATUS] Mock radar running");
            log::info!("Frame t={:.1}", t);

            // Drain logger buffer and forward to TUI
            let logs = utils_console::get_and_clear_log_lines();
            let _ = log_tx.send(logs);

            // Create moving points around player within bounds
            let mut points: Vec<RadarPoint> = Vec::new();
            for i in 0..20 {
                let a = t + (i as f32) * 0.31415926;
                let r = 100.0 + ((i * 47) as f32 % 300.0);
                let x = r * a.cos();
                let y = r * a.sin();
                let dz = (((i as i32) % 21) - 10) * 2;
                let health = (50 + ((i as i32 * 7 + (t as i32)) % 50)).max(1) as u32;
                points.push(RadarPoint { x, y, dz, health });
            }

            let yaw_deg = (t * 30.0) % 360.0;
            let frame = RadarFrame { yaw_deg, points };
            let _ = radar_tx.send(frame);

            t += 0.1;
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Run TUI until user quits with 'q'
    if let Err(e) = utils_console::run_tui(log_rx, radar_rx) {
        eprintln!("TUI error: {:?}", e);
    }
}
