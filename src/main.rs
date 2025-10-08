mod commands;
mod sci_frame_protocol;

use std::time::Duration;

use clap::{Error, Parser};
use serialport::SerialPort;

use crate::commands::command_storage_directory_size;

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
        port.write(&hello)?;
        let size = port.read(&mut read_buf)?;

        if size == 2 && read_buf.starts_with(&expected) {
            println!("Handshake sucessful");
            return Ok(());
        }
    }
}

fn main(){
    let args = CmdArgs::parse();

    let mut port = serialport::new(args.port, args.baud_rate)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    handshake(&mut port);
    //command_reset_device(&mut port, ResetType::Softreset);
    match command_storage_directory_size(&mut port) {
        Ok(size) => println!("Read size {size}"),
        Err(e) => println!("Error {e}"),
    }
}
