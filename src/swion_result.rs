// I hate this!
// Why are there so many different ways to encode this???

use strum::{Display, FromRepr};

#[derive(Debug, FromRepr, Display)]
#[repr(u8)]
pub enum SwionResult {
    Success,
    Error,
    DataNotFound,
    DataInvalid,
    DataIncomplete,
    AccessDenied,
    AuthentificationError,
    Locked,
    OutOfMemory,
    PermissionViolation,
    Busy,
    InvalidState,
    ChecksumMismatch,
    Closed,
    Abort,
    Timeout,
    NotSupported,
    ConstraintsViolation,
    Outdated,
    Exception,
    None,
}

pub fn parse_result_default(result: u8) -> SwionResult {
    SwionResult::from_repr(result).unwrap_or(SwionResult::None)
}

pub fn parse_result_var1(result: u8) -> SwionResult {
    match result {
        1 => SwionResult::Success,
        2 | 4 | 5 => SwionResult::AuthentificationError,
        3 => SwionResult::Locked,
        _ => SwionResult::Error,
    }
}
