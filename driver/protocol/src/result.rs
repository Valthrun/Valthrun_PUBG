use bitflags::bitflags;

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct CommandResult : u64 {
        const Error = 0x00;
        const Success = 0x01;

        const CommandInvalid = 0x10;
        const CommandParameterInvalid = 0x11;
        const CommandFeatureUnsupported = 0x12;
    }
}
