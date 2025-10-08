use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::{
    phoenix_encoding::decode_string,
    sci_frame_protocol::{decode_frame, encode_frame},
    types::{
        StorageBlockId, StorageBlockLength, StorageBlockOffset, StorageBlockPermissions,
        StorageBlockVersion, SwionResult,
    },
};

pub enum ResetType {
    Hardreset = 0,
    Softreset,
    BootupToHiddenMenu,
    BootupToTestMenu,
    BootupWithoutConfiguration,
    BootupToGsmTunnel,
    BootupToBootloader,
}

pub fn command_read_serial_number(
    port: &mut Box<dyn SerialPort>,
) -> Result<String, Box<dyn Error>> {
    let msg = [0x00, 0x03, 0x02];
    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(decode_string(&rsp[3..]))
}

pub fn command_reset_device(
    port: &mut Box<dyn SerialPort>,
    reset_type: ResetType,
) -> Result<(), Box<dyn Error>> {
    let msg = [0x01, 0x03, 0x00, reset_type as u8];
    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(())
}

#[derive(Debug)]
pub struct StorageBlockInfo {
    pub id: StorageBlockId,
    pub length: StorageBlockLength,
    pub version: StorageBlockVersion,
    pub permissions: StorageBlockPermissions,
}

pub fn command_storage_block_info(
    port: &mut Box<dyn SerialPort>,
    index: u16,
) -> Result<StorageBlockInfo, Box<dyn Error>> {
    let mut msg = [0x14, 0x03, 0x11, 0x00, 0x00];
    BigEndian::write_u16(&mut msg[3..], index);
    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(StorageBlockInfo {
        id: BigEndian::read_u16(&rsp[3..5]),
        length: BigEndian::read_u16(&rsp[5..7]),
        version: rsp[7],
        permissions: StorageBlockPermissions::from(rsp[8]),
    })
}

pub fn command_storage_directory_size(
    port: &mut Box<dyn SerialPort>,
) -> Result<u16, Box<dyn Error>> {
    let msg = [0x14, 0x03, 0x10];
    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(BigEndian::read_u16(&rsp[3..5]))
}

#[derive(Debug)]
pub struct PartialStorageBlock {
    id: StorageBlockId,
    offset: StorageBlockOffset,
    length: StorageBlockLength,
    result: SwionResult,
    data: Vec<u8>,
}

pub fn command_read_storage_block_partial(
    port: &mut Box<dyn SerialPort>,
    id: u16,
    offset: u16,
    length: u16,
) -> Result<PartialStorageBlock, Box<dyn Error>> {
    let mut msg = [0x14, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    BigEndian::write_u16(&mut msg[3..5], id);
    BigEndian::write_u16(&mut msg[5..7], offset);
    BigEndian::write_u16(&mut msg[7..9], length);
    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    let block = PartialStorageBlock {
        id: BigEndian::read_u16(&rsp[3..]),
        offset: BigEndian::read_u16(&rsp[5..]),
        length: BigEndian::read_u16(&rsp[7..]),
        result: SwionResult::from_repr(rsp[9]).unwrap_or(SwionResult::Error),
        data: Vec::from(&rsp[10..]),
    };

    Ok(block)
}
