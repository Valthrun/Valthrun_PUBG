use bitflags::bitflags;

use crate::utils;

pub type ProcessId = u32;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DriverFeature : u64 {
        const ProcessList               = 0x00_00_00_01;
        const ProcessModules            = 0x00_00_00_02;
        const ProcessProtectionKernel   = 0x00_00_00_04;
        const ProcessProtectionZenith   = 0x00_00_00_08;

        const MemoryRead                = 0x00_00_01_00;
        const MemoryWrite               = 0x00_00_02_00;

        const InputKeyboard             = 0x00_01_00_00;
        const InputMouse                = 0x00_02_00_00;

        const Metrics                   = 0x01_00_00_00;
        const DttExplicit               = 0x02_00_10_00;
        const CR3Sshenanigans           = 0x04_00_00_00;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessInfo {
    pub process_id: ProcessId,
    pub image_base_name: [u8; 0x0F],
    pub directory_table_base: u64,
}

impl ProcessInfo {
    pub fn get_image_base_name(&self) -> Option<&str> {
        utils::fixed_buffer_to_str(&self.image_base_name)
    }

    pub fn set_image_base_name(&mut self, value: &str) -> bool {
        utils::str_to_fixed_buffer(&mut self.image_base_name, value)
    }
}

impl Default for ProcessInfo {
    fn default() -> Self {
        Self {
            process_id: 0,
            image_base_name: [0; 0x0F],
            directory_table_base: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessModuleInfo {
    pub base_dll_name: [u8; 0x100],
    pub base_address: u64,
    pub module_size: u64,
}

impl ProcessModuleInfo {
    pub fn get_base_dll_name(&self) -> Option<&str> {
        utils::fixed_buffer_to_str(&self.base_dll_name)
    }

    pub fn set_base_dll_name(&mut self, value: &str) -> bool {
        utils::str_to_fixed_buffer(&mut self.base_dll_name, value)
    }
}

impl Default for ProcessModuleInfo {
    fn default() -> Self {
        Self {
            base_dll_name: [0; 0x100],
            base_address: 0,
            module_size: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryAccessResult {
    Success,
    PartialSuccess { bytes_copied: usize },

    ProcessUnknown,

    SourcePagedOut,
    DestinationPagedOut,
}

impl Default for MemoryAccessResult {
    fn default() -> Self {
        MemoryAccessResult::ProcessUnknown
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DirectoryTableType {
    /// Use the process directory table base specified by the system
    Default,

    /// Manually specify the directory table base for the target process
    Explicit { directory_table_base: u64 },

    /// Try to mitigate CR3 shenanigans and do not use the directory table base known to the system
    Cr3Shenanigans,
}
