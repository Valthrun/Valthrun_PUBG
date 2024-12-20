use core::ptr;

use crate::types::{
    DirectoryTableType,
    MemoryAccessResult,
    ProcessId,
};

#[derive(Debug, Clone, Copy)]
pub struct DriverCommandMemoryRead {
    pub process_id: ProcessId,
    pub directory_table_type: DirectoryTableType,

    pub address: u64,

    pub buffer: *mut u8,
    pub count: usize,

    pub result: MemoryAccessResult,
}

impl Default for DriverCommandMemoryRead {
    fn default() -> Self {
        Self {
            process_id: 0,
            directory_table_type: DirectoryTableType::Default,

            address: 0,

            buffer: ptr::null_mut(),
            count: 0,

            result: Default::default(),
        }
    }
}
