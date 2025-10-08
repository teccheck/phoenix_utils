use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::sci_frame_protocol::{decode_frame, encode_frame};

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
) -> Result<(), Box<dyn Error>> {
    let msg = [0x01, 0x03, 0x00, reset_type as u8];
    let frame = encode_frame(&msg);

    println!("Trying frame {:X?}", frame);

    port.write(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;

    let read_data = decode_frame(&read_buf[0..size]);
    println!("Decoded {:X?}", read_data);

    println!("Reset sucessful");
    return Ok(());
}


#[derive(Debug)]
pub struct StorageBlockInfo {
    id: u16,
    size: u16,
    version: u8,
    permissions: u8,
}

pub fn command_storage_block_info(port: &mut Box<dyn SerialPort>, index: u16) -> Result<StorageBlockInfo, Box<dyn Error>> {
    let mut msg = [0x14, 0x03, 0x11, 0x00, 0x00];
    BigEndian::write_u16(&mut msg[3..], index);
    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(StorageBlockInfo{
        id: BigEndian::read_u16(&rsp[3..5]),
        size: BigEndian::read_u16(&rsp[5..7]),
        version: rsp[7],
        permissions: rsp[8],
    })
}

pub fn command_storage_directory_size(
    port: &mut Box<dyn SerialPort>,
) -> Result<u16, Box<dyn Error>> {
    let msg = [0x14, 0x03, 0x10];
    let frame = encode_frame(&msg);
    port.write(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(BigEndian::read_u16(&rsp[3..5]))
}
