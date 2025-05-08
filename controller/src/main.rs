use std::{
    cell::RefCell,
    env,
    rc::Rc,
    sync::Arc,
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
use utils_common::get_os_info;
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
    utils_console::init().expect("Failed to initialize logger");
    if let Err(err) = real_main() {
        utils_console::show_critical_error(&format!("{:#}", err));
        utils_console::flush_frame_logs();
    }
}

fn real_main() -> anyhow::Result<()> {
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
        "{} v{} ({}). {}.",
        obfstr!("Valthrun_PUBG"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        platform_info
    );
    log::info!("{} {}", obfstr!("Build time:"), env!("BUILD_TIME"));

    let pubg = match PubgHandle::create(false) {
        Ok(pubg) => pubg,
        Err(err) => {
            if let Some(err) = err.downcast_ref::<InterfaceError>() {
                if let Some(detailed_message) = err.detailed_message() {
                    utils_console::show_critical_error(&detailed_message);
                    utils_console::flush_frame_logs();
                    return Ok(());
                }
            }

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

    log::info!("App initialized.");

    let mut update_fail_count = 0;
    let mut update_timeout: Option<(Instant, Duration)> = None;

    loop {
        if let Some((timeout, target)) = &update_timeout {
            if timeout.elapsed() > *target {
                update_timeout = None;
            } else {
                // On timeout, just skip the update
                continue;
            }
        }

        // Update game state
        if let Err(err) = app.borrow_mut().update() {
            if update_fail_count >= 10 {
                log::error!(
                    "Failed to update app for 10 times. Waiting for 1 second: {}",
                    err
                );
                log::error!("Last error: {:#}", err);

                update_timeout = Some((Instant::now(), Duration::from_secs(1)));
                update_fail_count = 0;
                utils_console::flush_frame_logs();
                continue;
            } else {
                update_fail_count += 1;
            }
        }

        // Update display
        utils_console::flush_frame_logs();

        // Control frame rate
        thread::sleep(Duration::from_millis(50));
    }
}
