use core::ptr;

use crate::types::{
    DirectoryTableType,
    ProcessId,
    ProcessInfo,
    ProcessModuleInfo,
};

#[derive(Debug, Clone, Copy)]
pub struct DriverCommandProcessList {
    /// In:  
    /// Number of elements `buffer` can hold
    pub buffer_capacity: usize,

    /// In/Out:  
    /// A pointer to a buffer with a capacity of at least `buffer_capacity` [ProcessInfo] entries.
    pub buffer: *mut ProcessInfo,

    /// Out:
    /// Total number of processes.
    /// If this number is greater then `buffer_capacity` the response is truncated.
    pub process_count: usize,
}

impl Default for DriverCommandProcessList {
    fn default() -> Self {
        Self {
            buffer: ptr::null_mut(),
            buffer_capacity: 0,

            process_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DriverCommandProcessModules {
    /// In:  
    /// Process id of the target
    pub process_id: ProcessId,

    /// In:  
    /// Type of the directory tabe to use when iterating the processes modules.
    pub directory_table_type: DirectoryTableType,

    /// In:  
    /// Number of elements `buffer` can hold
    pub buffer_capacity: usize,

    /// In/Out:  
    /// A pointer to a buffer with a capacity of at least `buffer_capacity` [ProcessModuleInfo] entries.
    pub buffer: *mut ProcessModuleInfo,

    /// Out:
    /// Total number of module.
    /// If this number is greater then `buffer_capacity` the response is truncated.
    pub module_count: usize,

    /// Out:
    /// If true the process is unknown / can not be resolved
    pub process_unknown: bool,
}

impl Default for DriverCommandProcessModules {
    fn default() -> Self {
        Self {
            process_id: 0,
            directory_table_type: DirectoryTableType::Default,

            buffer_capacity: 0,
            buffer: ptr::null_mut(),

            module_count: 0,
            process_unknown: true,
        }
    }
}
