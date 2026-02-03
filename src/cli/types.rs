use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
#[repr(u8)]
pub enum LedMode {
    Normal = 0xfe,
    On = 0xff,
    Off = 0,
    Red = 1,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
#[repr(u8)]
pub enum BacklightMode {
    Normal = 0xff,
    On = 0x0f,
    Off = 0,
}