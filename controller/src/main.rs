use std::{
    cell::RefCell,
    env,
    rc::Rc,
    sync::{
        mpsc,
        Arc,
    },
    thread,
    time::{
        Duration,
        Instant,
    },
};

use enhancements::Enhancement;
use obfstr::obfstr;
use pubg::{
    InterfaceError,
    PubgHandle,
    StatePubgHandle,
    StatePubgMemory,
};
use ratatui::text::Line;
use utils_common::get_os_info;
use utils_console;
use utils_state::StateRegistry;

use crate::enhancements::PlayerSpyer;

mod enhancements;

pub struct UpdateContext<'a> {
    pub states: &'a StateRegistry,
}

pub struct Application {
    pub states: StateRegistry,

    pub pubg: Arc<PubgHandle>,
    pub enhancements: Vec<Rc<RefCell<dyn Enhancement>>>,

    pub frame_read_calls: usize,
}

impl Application {
    fn update(&mut self) -> anyhow::Result<()> {
        self.states.invalidate_states();

        let update_context = UpdateContext {
            states: &self.states,
        };

        for enhancement in &self.enhancements {
            let mut enhancement = enhancement.borrow_mut();
            enhancement.update(&update_context)?;
        }

        let read_calls = self.pubg.ke_interface.total_read_calls();
        self.frame_read_calls = read_calls;

        Ok(())
    }
}

fn main() {
    if let Err(e) = utils_console::init_logger() {
        eprintln!("Failed to initialize logger: {:?}", e);
        return;
    }

    let (log_sender, log_receiver) = mpsc::channel::<Vec<Line<'static>>>();

    let app_thread_handle = thread::spawn(move || {
        if let Err(err) = app_logic_thread(log_sender) {
            log::error!("Critical error in app logic thread: {:#}", err);
        }
    });

    if let Err(e) = utils_console::run_tui(log_receiver) {
        log::error!("TUI exited with an error: {:?}", e);
    }

    if let Err(e) = app_thread_handle.join() {
        log::error!("App logic thread panicked: {:?}", e);
    }

    log::info!("Application terminated.");
}

fn app_logic_thread(log_sender: mpsc::Sender<Vec<Line<'static>>>) -> anyhow::Result<()> {
    let os_info = get_os_info()?;

    let platform_info = if os_info.is_windows {
        format!("Windows build {}", os_info.build_number)
    } else {
        format!(
            "Linux kernel {}.{}.{}",
            os_info.major_version, os_info.minor_version, os_info.build_number
        )
    };

    log::info!(
        "[STATUS] {} v{} ({}). {}.",
        obfstr!("Valthrun_PUBG"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        platform_info
    );
    log::info!("[STATUS] {} {}", obfstr!("Build time:"), env!("BUILD_TIME"));

    let initial_logs = utils_console::get_and_clear_log_lines();
    if !initial_logs.is_empty() {
        if log_sender.send(initial_logs).is_err() {
            log::warn!("Failed to send initial logs to TUI; TUI might have closed early.");
        }
    }

    let pubg = match PubgHandle::create(false) {
        Ok(pubg) => pubg,
        Err(err) => {
            log::error!("Failed to create PubgHandle: {:#}", err);
            if let Some(interface_err) = err.downcast_ref::<InterfaceError>() {
                if interface_err.detailed_message().is_some() {
                    // Detailed message already logged.
                }
            }
            let logs = utils_console::get_and_clear_log_lines();
            log_sender.send(logs).map_err(|se| {
                anyhow::anyhow!("Failed to send PubgHandle error logs to TUI: {}", se)
            })?;
            return Err(err);
        }
    };

    pubg.add_metrics_record(obfstr!("Valthrun_PUBG-status"), "initializing");

    let mut states = StateRegistry::new(1024 * 8);
    states.set(StatePubgHandle::new(pubg.clone()), ())?;
    states.set(StatePubgMemory::new(pubg.create_memory_view()), ())?;

    let app = Application {
        states,
        pubg: pubg.clone(),
        enhancements: vec![Rc::new(RefCell::new(PlayerSpyer {}))],
        frame_read_calls: 0,
    };
    let app = Rc::new(RefCell::new(app));

    pubg.add_metrics_record(
        obfstr!("Valthrun_PUBG-status"),
        &format!(
            "initialized, vesion: {}, git-hash: {}, platform: {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            platform_info
        ),
    );

    log::info!("[STATUS] App initialized.");

    let mut update_fail_count = 0;
    let mut update_timeout: Option<(Instant, Duration)> = None;

    loop {
        if let Some((timeout, target)) = &update_timeout {
            if timeout.elapsed() > *target {
                update_timeout = None;
            } else {
                thread::sleep(Duration::from_millis(50));
                let logs = utils_console::get_and_clear_log_lines();
                if !logs.is_empty() {
                    if log_sender.send(logs).is_err() {
                        log::info!("App logic: TUI closed, exiting loop.");
                        break;
                    }
                }
                continue;
            }
        }

        if let Err(err) = app.borrow_mut().update() {
            log::error!("App update failed: {:#}", err);
            if update_fail_count >= 10 {
                log::error!("Failed to update app for 10 times. Waiting for 1 second.");
                update_timeout = Some((Instant::now(), Duration::from_secs(1)));
                update_fail_count = 0;
            } else {
                update_fail_count += 1;
            }
        } else {
            update_fail_count = 0;
        }

        let logs = utils_console::get_and_clear_log_lines();
        if log_sender.send(logs).is_err() {
            log::info!("App logic: TUI closed, exiting loop.");
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
