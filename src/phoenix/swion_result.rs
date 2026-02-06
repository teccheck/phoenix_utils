// I hate this!
// Why are there so many different ways to encode this???
// There's enough space to store 256 different codes in a byte.
// So there's no reason to have such overlap!

use std::{error::Error, fmt};

use strum::{Display, FromRepr};

#[derive(Debug, FromRepr, Display, Clone)]
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

impl SwionResult {
    pub fn parse_default(result: u8) -> SwionResult {
        SwionResult::from_repr(result).unwrap_or(SwionResult::None)
    }

    pub fn parse_simple_inv(result: u8) -> SwionResult {
        match result {
            1 => SwionResult::Success,
            _ => SwionResult::Error,
        }
    }

    pub fn parse_auth(result: u8) -> SwionResult {
        match result {
            1 => SwionResult::Success,
            2 | 4 | 5 => SwionResult::AuthentificationError,
            3 => SwionResult::Locked,
            _ => SwionResult::Error,
        }
    }

    // Class: ac6
    pub fn parse_transaction(result: u8) -> SwionResult {
        match result {
            0 => SwionResult::Success,
            6 => SwionResult::Busy,
            7 => SwionResult::InvalidState,
            8 => SwionResult::DataInvalid,
            9 => SwionResult::ChecksumMismatch,
            _ => SwionResult::Error,
        }
    }

    // Class: abp
    pub fn parse_audio1(result: u8) -> SwionResult {
        match result {
            0 => SwionResult::Success,
            1 => SwionResult::DataInvalid,
            3 => SwionResult::OutOfMemory,
            4 => SwionResult::DataInvalid,
            _ => SwionResult::Error,
        }
    }

    // Class:abn
    pub fn parse_audio2(result: u8) -> SwionResult {
        match result {
            0 => SwionResult::Success,
            2 | 3 => SwionResult::DataInvalid,
            _ => SwionResult::Error,
        }
    }

    pub fn is_error(&self) -> bool {
        !matches!(self, Self::Success)
    }
}

#[derive(Debug, Clone)]
pub struct SwionError {
    operation: String,
    reason: SwionResult,
}

impl fmt::Display for SwionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} unsuccessful: {}", self.operation, self.reason)
    }
}

impl Error for SwionError {}

impl SwionError {
    pub fn new(operation: String, reason: SwionResult) -> SwionError {
        SwionError { operation, reason }
    }
}
