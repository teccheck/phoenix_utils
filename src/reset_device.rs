use clap::Error;
use serialport::SerialPort;

use crate::sci_frame_protocol::encode_frame;

pub enum ResetType {
    Hardreset = 0,
    Softreset,
    BootupToHiddenMenu,
    BootupToTestMenu,
    BootupWithoutConfiguration,
    BootupToGsmTunnel,
    BootupToBootloader,
}

pub fn command_reset_device(
    port: &mut Box<dyn SerialPort>,
    reset_type: ResetType,
) -> Result<(), Error> {
    let msg = [0x01, 0x03, 0x00, reset_type as u8];
    let frame = encode_frame(&msg);

    println!("Trying frame {:X?}", frame);

    port.write(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;

    println!("Reset sucessful");
    return Ok(());
}
