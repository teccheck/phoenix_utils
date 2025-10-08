mod commands;
mod phoenix_encoding;
mod sci_frame_protocol;
mod tasks;
mod types;

use std::time::Duration;

use clap::{Error, Parser};
use serialport::SerialPort;

use crate::commands::{
    StorageBlockInfo, command_read_feature_flags_available, command_read_feature_flags_enabled,
    command_read_firmware_version, command_read_serial_number, command_read_unique_id,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CmdArgs {
    #[arg(short, long, help = "Serial port to use (eg. /dev/ttyUSB0)")]
    port: String,

    #[arg(short, long, default_value_t = 57600, help = "Baud rate (eg. 57600)")]
    baud_rate: u32,
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

fn print_storage_dir(dir: Vec<StorageBlockInfo>) {
    println!("| ID   | Version | Size   | Flags |");

    for block in dir {
        println!(
            "| {:>4x} | {:>7} | {:>6} | {:>5} |",
            block.id,
            block.version,
            block.length,
            block.permissions.flag_string()
        );
    }
}

fn main() {
    let args = CmdArgs::parse();

    let mut port = serialport::new(args.port, args.baud_rate)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let _ = handshake(&mut port);

    match command_read_serial_number(&mut port) {
        Ok(r) => println!("SN: {}", r),
        Err(e) => println!("Error: {e}"),
    }

    match command_read_firmware_version(&mut port) {
        Ok(r) => println!("Firm: {}", r),
        Err(e) => println!("Error: {e}"),
    }

    match command_read_feature_flags_available(&mut port) {
        Ok(r) => println!("Flags available: {r:b}"),
        Err(e) => println!("Error: {e}"),
    }

    match command_read_feature_flags_enabled(&mut port) {
        Ok(r) => println!("Flags enabled: {r:b}"),
        Err(e) => println!("Error: {e}"),
    }

    match command_read_unique_id(&mut port) {
        Ok(r) => println!("Unique ID: {:x?}", r),
        Err(e) => println!("Error: {e}"),
    }
}
