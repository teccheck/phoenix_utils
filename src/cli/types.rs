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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
#[repr(u8)]
pub enum PagerKey {
    NavDown = 1,
    NavUp = 2,
    Enter = 4,
    // Not sure about this one.
    // Only present on s.QUAD models
    //Back = 8,
}
