mod commands;
mod types;

use std::time::Duration;

use clap::{Parser, Subcommand};
use clap_num::maybe_hex;

use crate::{cli::{commands::{backlight_mode, led_mode, write_feature_flags}, types::{BacklightMode, LedMode}}, phoenix::{
    commands::{command_bootup_device, command_reset_device, command_shutdown_device},
    raw_serial_protocol::handshake,
    tasks::{
        debug_task, task_dump_storage, task_print_cra_capabilities, task_print_device_info,
        task_print_storage_block, task_print_storage_directory, task_reset_password,
        task_set_password, task_try_authenticate,
    },
    types::{ResetType, StorageBlockId, StorageBlockLength, StorageBlockOffset},
}};

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

    #[arg(long, help = "SHA1 hased programming password")]
    auth_hash: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Reboot {
        #[arg(short, long, value_enum, default_value_t = ResetType::Softreset)]
        reboot_type: ResetType,
    },
    Bootup,
    Shutdown,
    PrintStorageDir,
    DumpStorage,
    ReadStorageBlock {
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockId>)]
        id: StorageBlockId,
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockOffset>, default_value_t=0)]
        offset: StorageBlockOffset,
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockLength>)]
        length: StorageBlockLength,
    },
    WriteFeatureFlags {
        flags: Vec<String>,
    },
    CRAReadCapabilities,
    ResetPassword,
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

    Debug {
        #[arg(value_parser=maybe_hex::<u16>)]
        command_type: u16,

        #[arg(long, help = "Args data")]
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
        task_print_device_info(&mut port);
    }

    match task_try_authenticate(&mut port, args.auth, args.auth_hash) {
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
            Commands::Reboot { reboot_type } => command_reset_device(&mut port, reboot_type),
            Commands::Shutdown => command_shutdown_device(&mut port),
            Commands::Bootup => command_bootup_device(&mut port),
            Commands::PrintStorageDir => task_print_storage_directory(&mut port),
            Commands::DumpStorage => task_dump_storage(&mut port),
            Commands::ReadStorageBlock { id, offset, length } => {
                task_print_storage_block(&mut port, id, offset, length)
            }
            Commands::WriteFeatureFlags { flags } => write_feature_flags(&mut port, flags),
            Commands::CRAReadCapabilities => task_print_cra_capabilities(&mut port),
            Commands::ResetPassword => task_reset_password(&mut port),
            Commands::SetPassword { password } => task_set_password(&mut port, password),
            Commands::Debug { command_type, data } => debug_task(&mut port, command_type, data),
            Commands::Led { mode } => led_mode(&mut port, mode),
            Commands::Backlight { mode } => backlight_mode(&mut port, mode),
        };

        match result {
            Ok(_) => println!("Successful"),
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
