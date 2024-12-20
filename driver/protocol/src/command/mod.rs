mod init;
pub use init::*;

mod process;
pub use process::*;

mod memory_read;
pub use memory_read::*;

mod memory_write;
pub use memory_write::*;

mod metrics;
pub use metrics::*;

mod input_keyboard;
pub use input_keyboard::*;

mod input_mouse;
pub use input_mouse::*;

mod process_protection;
pub use process_protection::*;

mod cr3;
pub use cr3::*;

pub trait DriverCommand: Default + Copy {
    const COMMAND_ID: u32;
}

macro_rules! define_command {
    ($struct:ty, $id:expr) => {
        impl DriverCommand for $struct {
            const COMMAND_ID: u32 = $id;
        }
    };
}

/* this command id should always be the same */
define_command!(DriverCommandInitialize, 0x00);

define_command!(DriverCommandProcessList, 0x01);
define_command!(DriverCommandProcessModules, 0x02);
define_command!(DriverCommandMemoryRead, 0x03);
define_command!(DriverCommandMemoryWrite, 0x04);

define_command!(DriverCommandInputKeyboard, 0x05);
define_command!(DriverCommandInputMouse, 0x06);
define_command!(DriverCommandMetricsReportSend, 0x07);
define_command!(DriverCommandProcessProtection, 0x08);

define_command!(DriverCommandCr3ShenanigansEnable, 0x09);
define_command!(DriverCommandCr3ShenanigansDisable, 0x0A);
