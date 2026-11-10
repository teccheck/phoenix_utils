mod commands;
mod types;
mod utils;

use clap::{Parser, Subcommand};
use clap_num::maybe_hex;

use phoenix::{
    self,
    protocols::raw_serial_protocol::{init_connection, try_init_connection},
    types::{
        ResetType, SERIAL_CONFIG_1, SERIAL_CONFIG_2, StorageBlockId, StorageBlockLength,
        StorageBlockOffset,
    },
};
use serialport::SerialPortType;

use crate::{
    commands::{
        backlight_mode, cra_read_capabilities, debug, feature_flags_read_enabled,
        feature_flags_read_supported, feature_flags_read_unique_id, feature_flags_write, key_press,
        key_release, led_mode, print_device_info, print_storage_block, print_storage_directory,
        time_get, time_set, write_storage_block,
    },
    types::{BacklightMode, DisplayMode, LedMode, PagerKey},
    utils::decode_hex,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CmdArgs {
    #[arg(short, long, help = "Serial port to use")]
    port: Option<String>,

    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Serial config to use. 0 for auto, 1 for newer, 2 for older devices"
    )]
    serial_config: u8,

    #[arg(short, long, default_value_t = false, help = "Show device info")]
    info: bool,

    #[arg(short, long, help = "Programming password")]
    auth: Option<String>,

    #[arg(
        long,
        help = "SHA1 hased programming password as hex string without spaces"
    )]
    auth_hash: Option<String>,

    #[arg(
        short,
        long,
        help = "Enables update mode for use if the device is in update mode. In update mode, no handshake is made. Only works together with the serial_config option",
        default_value_t = false
    )]
    update: bool,

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

    /// Write data to a specific storage block. Be careful!
    WriteStorageBlock {
        #[arg(value_parser=maybe_hex::<StorageBlockId>)]
        id: StorageBlockId,
        #[arg(value_parser=maybe_hex::<StorageBlockOffset>, default_value_t=0)]
        offset: StorageBlockOffset,
        #[arg(value_parser=maybe_hex::<StorageBlockLength>)]
        length: StorageBlockLength,
        #[arg(help = "Data to write as hex string without spaces (Example: E100)")]
        data: String,
    },

    /// Get all currently enabled features
    FlagsReadEnabled,

    /// Get all features that can be enabled
    FlagsReadSupported,

    /// Write feature flags to the device (does not add them, but replaces the previous value)
    FlagsWrite {
        /// Flags to be enabled, seperated by spaces.
        flags: Vec<String>,
    },

    /// Reads some random value. Not sure what that's for.
    FlagsReadUniqueId,

    /// Read all command families this device supports
    CRAReadCapabilities,

    /// Reset the devices programming password (only on firmware < 4)
    ResetPassword,

    /// Set a new programming password (only on firmware < 4)
    SetPassword {
        password: String,
    },

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
        mode: DisplayMode,
    },

    /// Set the time on the device
    TimeSet {
        #[arg(help = "Set a custom UTC time. Format: 2012-01-30T15:30:59")]
        time: Option<String>,
    },

    /// Get the time on the device
    TimeGet {
        #[arg(short, long, default_value_t = false, help = "Get time as UTC")]
        utc: bool,
    },

    /// Start firmware update mode
    StartFirmwareUpdate {},

    UpdateReset {},

    UpdateFrameTest {},

    /// Try out an arbitrary command code with optional data.
    /// Use carefully! Prepare for unforeseen consequences λ.
    Debug {
        #[arg(value_parser=maybe_hex::<u16>)]
        command_type: u16,

        #[arg(
            short,
            long,
            help = "Args data as hex string without spaces (Example: E100)"
        )]
        data: Option<String>,

        #[arg(short, long, help = "Try to decode output as string")]
        string_decode: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CmdArgs::parse();

    println!("Welcome to Phoenix Utils");
    println!("Use --help to see what commands are available.");
    println!();

    let Some(port_path) = args.port.or_else(|| {
        let available = serialport::available_ports().ok()?;
        let first = available.iter().find(|p| p.port_type != SerialPortType::Unknown)?;
        Some(first.port_name.clone())
    }) else {
        println!("No port found");
        return Ok(());
    };

    println!("Trying port {}", port_path);

    let (mut port, device_type) = match args.serial_config {
        1 => try_init_connection(&port_path, SERIAL_CONFIG_1, !args.update)?,
        2 => try_init_connection(&port_path, SERIAL_CONFIG_2, !args.update)?,
        _ => init_connection(&port_path)?,
    };

    if !args.update {
        println!("Device Type: {:?}", device_type);

        if args.info {
            print_device_info(&mut port);
        }

        let auth_hash = if let Some(s) = args.auth_hash {
            Some(decode_hex(s.as_str())?)
        } else {
            None
        };

        match phoenix::tasks::try_authenticate(&mut port, args.auth, auth_hash) {
            Ok(_) => {
                println!("Auth successful");
            }
            Err(e) => {
                println!("Auth error: {}", e);
                return Err(e);
            }
        }

        println!();
    }

    if let Some(command) = args.command {
        let result = match command {
            Commands::Reboot { reboot_type } => {
                phoenix::commands::boot::reboot(&mut port, reboot_type)
            }
            Commands::Shutdown => phoenix::commands::boot::shutdown(&mut port),
            Commands::Bootup => phoenix::commands::boot::startup(&mut port),
            Commands::PrintStorageDir => print_storage_directory(&mut port),
            Commands::DumpStorage => phoenix::tasks::dump_storage(&mut port),
            Commands::ReadStorageBlock { id, offset, length } => {
                print_storage_block(&mut port, id, offset, length)
            }
            Commands::WriteStorageBlock {
                id,
                offset,
                length,
                data,
            } => write_storage_block(&mut port, id, offset, length, &data),
            Commands::FlagsReadEnabled => feature_flags_read_enabled(&mut port),
            Commands::FlagsReadSupported => feature_flags_read_supported(&mut port),
            Commands::FlagsWrite { flags } => feature_flags_write(&mut port, flags),
            Commands::FlagsReadUniqueId => feature_flags_read_unique_id(&mut port),
            Commands::CRAReadCapabilities => cra_read_capabilities(&mut port),
            Commands::ResetPassword => phoenix::tasks::reset_password(&mut port),
            Commands::SetPassword { password } => phoenix::tasks::set_password(&mut port, &password),
            Commands::Debug {
                command_type,
                data,
                string_decode,
            } => debug(&mut port, command_type, data, string_decode),
            Commands::Led { mode } => led_mode(&mut port, mode),
            Commands::Backlight { mode } => backlight_mode(&mut port, mode),
            Commands::KeyPress { key } => key_press(&mut port, key),
            Commands::KeyRelease { key } => key_release(&mut port, key),
            Commands::KeyClick => phoenix::commands::tools::key_click(&mut port),
            Commands::Display { mode } => {
                phoenix::commands::display_test_mode(&mut port, mode as u8)
            }
            Commands::TimeSet { time } => time_set(&mut port, time),
            Commands::TimeGet { utc } => time_get(&mut port, utc),
            Commands::StartFirmwareUpdate {} => {
                phoenix::commands::sys::start_firmware_update(&mut port)
            }
            Commands::UpdateReset {} => phoenix::commands::update::reset(&mut port),
            Commands::UpdateFrameTest {} => phoenix::tasks::update_frame_test(&mut port),
        };

        match result {
            Ok(_) => {},
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
