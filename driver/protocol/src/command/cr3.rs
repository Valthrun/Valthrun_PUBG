#[derive(Default, Debug, Clone, Copy)]
pub struct DriverCommandCr3ShenanigansEnable {
    pub mitigation_strategy: u32,
    pub mitigation_flags: u32,

    pub success: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DriverCommandCr3ShenanigansDisable {}
