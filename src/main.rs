mod commands;
mod phoenix_encoding;
mod sci_frame_protocol;
mod tasks;
mod types;

use std::time::Duration;

use clap::{Error, Parser, Subcommand};
use serialport::SerialPort;

use crate::{
    commands::{command_bootup_device, command_reset_device, command_shutdown_device},
    tasks::{task_print_device_info, task_print_storage_directory},
    types::ResetType,
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
}

fn handshake(port: &mut Box<dyn SerialPort>) -> Result<(), Error> {
    let hello: [u8; 3] = [0x55, 0x7e, 0x55];
    let expected: [u8; 2] = [0x56, 0x56];

    let mut read_buf: [u8; 2] = [0; 2];

    loop {
        port.write_all(&hello)?;
        let size = port.read(&mut read_buf)?;

        if size == 2 && read_buf.starts_with(&expected) {
            println!("Handshake sucessful");
            return Ok(());
        }
    }
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

    let _ = handshake(&mut port);

    println!("");

    if args.info {
        task_print_device_info(&mut port);
    }

    if let Some(command) = args.command {
        match command {
            Commands::Reboot { reboot_type } => {
                let _ = command_reset_device(&mut port, reboot_type);
            }
            Commands::Shutdown => {
                let _ = command_shutdown_device(&mut port);
            }
            Commands::Bootup => {
                let _ = command_bootup_device(&mut port);
            }
            Commands::PrintStorageDir => {
                let _ = task_print_storage_directory(&mut port);
            }
        }
    };

    return;
}
