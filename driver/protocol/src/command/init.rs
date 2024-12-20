use crate::{
    types::DriverFeature,
    utils,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct VersionInfo {
    pub application_name: [u8; 0x20],
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
}

impl VersionInfo {
    pub fn get_application_name(&self) -> Option<&str> {
        utils::fixed_buffer_to_str(&self.application_name)
    }

    pub fn set_application_name(&mut self, value: &str) -> bool {
        utils::str_to_fixed_buffer(&mut self.application_name, value)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DriverCommandInitialize {
    /* The order of the first few fields should be consistent accross versions. */
    // In:
    pub client_protocol_version: u32,

    // Out:
    pub driver_protocol_version: u32,

    /* These fields are only valid to access if the client and driver protocol version are equal */
    // Out:
    pub result: InitializeResult,

    // In:
    pub client_version: VersionInfo,

    // Out:
    pub driver_version: VersionInfo,

    // Out:
    pub driver_features: DriverFeature,
}

#[derive(Debug, Clone, Copy)]
pub enum InitializeResult {
    Success,

    /// The driver is not available.
    /// E.g. the kernel component has not been mapped.
    Unavailable,
}

impl Default for InitializeResult {
    fn default() -> Self {
        Self::Unavailable
    }
}
