use std::
    time::{Duration, Instant}
;

use utils_console;

mod app;
mod enhancements;
mod settings;
mod utils;
mod view;

fn main() {
    if let Err(err) = real_main() {
        utils_console::show_critical_error(&format!("{:#}", err));
    }
}

fn real_main() -> anyhow::Result<()> {
    let (overlay, app) = app::initialize_app()?;

    //let mut update_fail_count = 0;
    let mut update_timeout: Option<(Instant, Duration)> = None;

    overlay.main_loop(
        {
            let app = app.clone();
            move |controller, window| {
                let mut app = app.borrow_mut();
                if let Err(err) = app.pre_update(controller, window) {
                    utils_console::show_critical_error(&format!("{:#}", err));
                    false
                } else {
                    true
                }
            }
        },
        move |ui, unicode_text| {
            let mut app = app.borrow_mut();

            if let Some((timeout, target)) = &update_timeout {
                if timeout.elapsed() > *target {
                    update_timeout = None;
                } else {
                    /* Not updating. On timeout... */
                    return true;
                }
            }

            if let Err(_) = app.update(ui) {
                /*if update_fail_count >= 10 {
                    log::error!("Over 10 errors occurred. Waiting 1s and try again.");
                    log::error!("Last error: {:#}", err);

                    update_timeout = Some((Instant::now(), Duration::from_millis(1000)));
                    update_fail_count = 0;
                    return true;
                } else {
                    update_fail_count += 1;
                }*/
            }

            app.render(ui, unicode_text);
            true
        },
    );

    Ok(())
}
