mod commands;
mod types;

use std::time::Duration;

use clap::{Parser, Subcommand};
use clap_num::maybe_hex;

use crate::{
    cli::{
        commands::{
            backlight_mode, cra_read_capabilities, feature_flags_read_enabled, feature_flags_read_supported, feature_flags_read_unique_id, feature_flags_write, key_press, key_release, led_mode, print_device_info, print_storage_block, print_storage_directory, time_get, time_set
        },
        types::{BacklightMode, DisplayMode, LedMode, PagerKey},
    },
    phoenix::{
        self,
        raw_serial_protocol::handshake,
        types::{ResetType, StorageBlockId, StorageBlockLength, StorageBlockOffset},
    },
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CmdArgs {
    #[arg(
        short,
        long,
        default_value = "/dev/ttyUSB0",
        help = "Serial port to use"
    )]
    port: String,

    #[arg(short, long, default_value_t = 57600, help = "Baud rate")]
    baud_rate: u32,

    #[arg(short, long, default_value_t = true, help = "Show device info")]
    info: bool,

    #[arg(short, long, help = "Programming password")]
    auth: Option<String>,

    #[arg(long, help = "SHA1 hased programming password as hex string without spaces")]
    auth_hash: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Reboot the device
    Reboot {
        #[arg(value_enum, default_value_t = ResetType::Softreset)]
        reboot_type: ResetType,
    },

    /// Start the device
    Bootup,

    /// Shut the device down
    Shutdown,

    /// Print storage blocks in directory with their metadata
    PrintStorageDir,

    /// Dump all storage blocks from directory into files
    DumpStorage,

    /// Read the specified storage block with length and offset
    ReadStorageBlock {
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockId>)]
        id: StorageBlockId,
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockOffset>, default_value_t=0)]
        offset: StorageBlockOffset,
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockLength>)]
        length: StorageBlockLength,
    },

    /// Get all currently enabled features
    FeatureFlagsReadEnabled,

    /// Get all features that can be enabled
    FeatureFlagsReadSupported,

    /// Write feature flags to the device (replaces the previous value)
    FeatureFlagsWrite { flags: Vec<String> },

    /// Reads some random value. Not sure what that's for.
    FeatureFlagsReadUniqueId,

    /// Read all command families this device supports
    CRAReadCapabilities,

    /// Reset the devices programming password (only on firmware < 4)
    ResetPassword,

    /// Set a new programming password (only on firmware < 4)
    SetPassword { password: String },

    /// Control the alarm LED (if there is one)
    Led {
        #[arg(value_enum)]
        mode: LedMode,
    },

    /// Control the display backlight
    Backlight {
        #[arg(value_enum)]
        mode: BacklightMode,
    },

    /// Send a virtual key press to the device
    KeyPress {
        #[arg(value_enum)]
        key: PagerKey,
    },

    /// Send a virtual key release to the device
    KeyRelease {
        #[arg(value_enum)]
        key: PagerKey,
    },

    /// Makes the key press click sound
    KeyClick,

    /// Set display test mode
    Display {
        #[arg(value_enum)]
        mode: DisplayMode
    },

    TimeSet {
        #[arg(help = "Set a custom UTC time. Format: 2012-01-30T15:30:59")]
        time: Option<String>,
    },

    TimeGet {
        #[arg(short, long, default_value_t = false, help = "Get time as UTC")]
        utc: bool,
    },

    /// Try out an arbitrary command code with optional data.
    /// Might have unforeseen consequences. Use carefully!
    Debug {
        #[arg(value_parser=maybe_hex::<u16>)]
        command_type: u16,

        #[arg(short, long, help = "Args data as hex string without spaces (Example: E100)")]
        data: Option<String>,
    },
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = CmdArgs::parse();

    let welcome = "#######################################################################
#######################################################################
###                                                                 ###
###    _______________       _____  ______   __  ___                ###
###   /  ___________  \\     |  __ \\|  ____| /_ |/ _ \\      /\\       ###
###   | |  /     \\  | |     | |  | | |__     | | | | |    /  \\      ###
###   | |  |() ()|  | |     | |  | |  __|    | | | | |   / /\\ \\     ###
###   | |  \\  ^  /  | |     | |__| | |____   | | |_| |  / ____ \\    ###
###   | |____|||____| |     |_____/|______|  |_|\\___/  /_/    \\_\\   ###
###   |               |                                             ###
###   |    #######    |                                             ###
###   |               |      _    _ _______ _____ _       _____     ###
###   |  /  \\         |     | |  | |__   __|_   _| |     / ____|    ###
###   |\\ \\  /    ( ) /|     | |  | |  | |    | | | |    | (___      ###
###   | \\           / |     | |  | |  | |    | | | |     \\___ \\     ###
###   |  \\     ( ) /  |     | |__| |  | |   _| |_| |____ ____) |    ###
###   \\___\\_______/___/      \\____/   |_|  |_____|______|_____/     ###
###                                                                 ###
###                                                                 ###
#######################################################################
#######################################################################";

    println!("{welcome}\n");
    println!(
        "Trying port {} with baud rate {}",
        args.port, args.baud_rate
    );

    let mut port = serialport::new(args.port, args.baud_rate)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(1000))
        .open()?;

    let device_type = handshake(&mut port)?;
    println!("Device Type: {:?}", device_type);

    if args.info {
        print_device_info(&mut port);
    }

    match phoenix::tasks::try_authenticate(&mut port, args.auth, args.auth_hash) {
        Ok(_) => {
            println!("Auth successful");
        }
        Err(e) => {
            println!("Auth error: {}", e);
            return Err(e);
        }
    }

    if let Some(command) = args.command {
        let result = match command {
            Commands::Reboot { reboot_type } => {
                phoenix::commands::device_reset::reboot(&mut port, reboot_type)
            }
            Commands::Shutdown => phoenix::commands::device_reset::shutdown(&mut port),
            Commands::Bootup => phoenix::commands::device_reset::startup(&mut port),
            Commands::PrintStorageDir => print_storage_directory(&mut port),
            Commands::DumpStorage => phoenix::tasks::dump_storage(&mut port),
            Commands::ReadStorageBlock { id, offset, length } => {
                print_storage_block(&mut port, id, offset, length)
            }
            Commands::FeatureFlagsReadEnabled => feature_flags_read_enabled(&mut port),
            Commands::FeatureFlagsReadSupported => feature_flags_read_supported(&mut port),
            Commands::FeatureFlagsWrite { flags } => feature_flags_write(&mut port, flags),
            Commands::FeatureFlagsReadUniqueId => feature_flags_read_unique_id(&mut port),
            Commands::CRAReadCapabilities => cra_read_capabilities(&mut port),
            Commands::ResetPassword => phoenix::tasks::reset_password(&mut port),
            Commands::SetPassword { password } => phoenix::tasks::set_password(&mut port, password),
            Commands::Debug { command_type, data } => {
                phoenix::tasks::debug_task(&mut port, command_type, data)
            }
            Commands::Led { mode } => led_mode(&mut port, mode),
            Commands::Backlight { mode } => backlight_mode(&mut port, mode),
            Commands::KeyPress { key } => key_press(&mut port, key),
            Commands::KeyRelease { key } => key_release(&mut port, key),
            Commands::KeyClick => phoenix::commands::tools::key_click(&mut port),
            Commands::Display { mode } => phoenix::commands::display_test_mode(&mut port, mode as u8),
            Commands::TimeSet { time } => time_set(&mut port, time),
            Commands::TimeGet { utc } => time_get(&mut port, utc),
        };

        match result {
            Ok(_) => println!("Successful"),
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
