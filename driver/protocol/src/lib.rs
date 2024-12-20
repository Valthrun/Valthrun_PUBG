#![no_std]

mod result;
pub use result::*;

pub mod utils;

pub mod types;

pub mod command;

pub const PROTOCOL_VERSION: u32 = 0x03;

pub type FnCommandHandler = unsafe extern "C" fn(
    // the id of the command to handle
    command_id: u32,

    // payload and payload length of the command
    payload: *mut u8,
    payload_length: usize,

    // buffer for retriving the error message if an error occurrs
    error_message: *mut u8,
    error_message_length: usize,
) -> u64; /* CommandResult */
