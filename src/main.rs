mod commands;
mod phoenix_encoding;
mod sci_frame_protocol;
mod swion_result;
mod tasks;
mod types;

use std::time::Duration;

use clap::{Error, Parser, Subcommand};
use clap_num::maybe_hex;
use serialport::SerialPort;

use crate::{
    commands::{command_bootup_device, command_reset_device, command_shutdown_device},
    tasks::{
        task_print_cra_capabilities, task_print_device_info, task_print_storage_block,
        task_print_storage_directory, task_try_authenticate,
    },
    types::{DeviceType, ResetType, StorageBlockId, StorageBlockLength, StorageBlockOffset},
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

    #[arg(long, help = "SHA1 hased programming password")]
    auth_hash: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Reboot {
        #[arg(short, long, value_enum, default_value_t = ResetType::Softreset)]
        reboot_type: ResetType,
    },
    Bootup,
    Shutdown,
    PrintStorageDir,
    ReadStorageBlock {
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockId>)]
        id: StorageBlockId,
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockOffset>, default_value_t=0)]
        offset: StorageBlockOffset,
        #[arg(short, long, value_parser=maybe_hex::<StorageBlockLength>)]
        length: StorageBlockLength,
    },
    CRAReadCapabilities,
}

fn handshake(port: &mut Box<dyn SerialPort>) -> Result<DeviceType, Error> {
    let hello: [u8; 3] = [0x55, 0x7e, 0x55];
    let expected: [u8; 2] = [0x56, 0x56];

    let mut read_buf: [u8; 2] = [0; 2];

    loop {
        port.write_all(&hello)?;
        let size = port.read(&mut read_buf)?;

        if size == 2 && read_buf.starts_with(&expected) {
            println!("Handshake sucessful");
            break;
        }
    }

    let device_tpye_cmd: [u8; 1] = [0x55];
    port.write_all(&device_tpye_cmd)?;
    let size = port.read(&mut read_buf)?;

    let device_type = match read_buf[0] {
        0x55 => DeviceType::B,
        0x56 => DeviceType::DE10A,
        0x57 => DeviceType::D,
        0x58 => DeviceType::E,
        0x59 => DeviceType::F,
        _ => DeviceType::A,
    };

    return Ok(device_type);
}

fn main() {
    let args = CmdArgs::parse();

    println!("Welcome to DE10A Utils");
    println!(
        "Trying port {} with baud rate {}",
        args.port, args.baud_rate
    );

    let mut port = serialport::new(args.port, args.baud_rate)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let device_type = handshake(&mut port);
    println!("Device Type: {:?}", device_type);

    println!("");

    if args.info {
        task_print_device_info(&mut port);
    }

    match task_try_authenticate(&mut port, args.auth, args.auth_hash) {
        Ok(_) => {
            println!("Auth successful");
        },
        Err(e) => {
            println!("Auth error: {}", e);
            return;
        },
    }

    if let Some(command) = args.command {
        let result = match command {
            Commands::Reboot { reboot_type } => {
                command_reset_device(&mut port, reboot_type)
            }
            Commands::Shutdown => {
                command_shutdown_device(&mut port)
            }
            Commands::Bootup => {
                command_bootup_device(&mut port)
            }
            Commands::PrintStorageDir => {
                task_print_storage_directory(&mut port)
            }
            Commands::ReadStorageBlock { id, offset, length } => {
                task_print_storage_block(&mut port, id, offset, length)
            }
            Commands::CRAReadCapabilities => {
                task_print_cra_capabilities(&mut port)
            }
        };

        match result {
            Ok(_) => println!("Successful"),
            Err(e) => println!("Error: {}", e),
        }
    }

    return;
}
