use bitmask_enum::bitmask;
use strum::FromRepr;

#[derive(Debug, FromRepr)]
#[repr(u8)]
pub enum SwionResult {
    Success = 0,
    Error = 1,
    DataNotFound = 2,
    DataInvalid = 3,
    DataIncomplete = 4,
    AccessDenied = 5,
}

pub type StorageBlockId = u16;
pub type StorageBlockOffset = u16;
pub type StorageBlockLength = u16;
pub type StorageBlockVersion = u8;

#[bitmask(u8)]
pub enum StorageBlockPermissions {
    Read,
    Write,
    I,
    P,
}

impl StorageBlockPermissions {
    pub fn flag_string(&self) -> String {
        let read = if self.contains(StorageBlockPermissions::Read){"R"} else {"-"};
        let write = if self.contains(StorageBlockPermissions::Write){"W"} else {"-"};
        let i = if self.contains(StorageBlockPermissions::I){"I"} else {"-"};
        let p = if self.contains(StorageBlockPermissions::P){"P"} else {"-"};
        return format!("{}{}{}{}", read, write, i, p);
    }
}
