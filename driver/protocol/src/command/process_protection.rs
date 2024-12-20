#[derive(Debug, Clone, Copy)]
pub enum ProcessProtectionMode {
    None,
    Kernel,
    Zenith,
}

impl Default for ProcessProtectionMode {
    fn default() -> Self {
        ProcessProtectionMode::None
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DriverCommandProcessProtection {
    pub mode: ProcessProtectionMode,
}
