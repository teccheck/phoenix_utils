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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
#[repr(u8)]
pub enum DisplayMode {
    /// Normal mode. Only takes effect if the display is updated. For example by pressing a button.
    Normal = 0,
    /// All pixels on
    On = 1,
    /// All pixels off
    Off = 2,
    /// Checkerboard pattern
    Checkerboard = 3,
    /// Inverse checkerboard pattern
    CheckerboardInv = 4,
    /// Not supported on all models
    Grayscale = 5,
}
