use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::phoenix::{
    commands::send_command,
    swion_result::{SwionError, SwionResult},
    types::{
        CommandType, PartialStorageBlock, StorageBlockId, StorageBlockInfo, StorageBlockPermissions,
    },
};

pub fn delete_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
) -> Result<(), Box<dyn Error>> {
    let mut data = [2, 0_u8];
    BigEndian::write_u16(&mut data, id);
    let rsp = send_command(port, CommandType::StorageDeleteBlock, &data)?;
    println!("rsp: {:x?}", rsp);

    Ok(())
}

pub fn read_block_info(
    port: &mut Box<dyn SerialPort>,
    index: u16,
) -> Result<StorageBlockInfo, Box<dyn Error>> {
    let mut data = [2, 0_u8];
    BigEndian::write_u16(&mut data, index);
    let rsp = send_command(port, CommandType::StorageReadBlockInfo, &data)?;

    Ok(StorageBlockInfo {
        id: BigEndian::read_u16(&rsp[3..5]),
        length: BigEndian::read_u16(&rsp[5..7]),
        version: rsp[7],
        permissions: StorageBlockPermissions::from(rsp[8]),
    })
}

pub fn read_dir_size(port: &mut Box<dyn SerialPort>) -> Result<u16, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::StorageReadDirSize, &[])?;
    Ok(BigEndian::read_u16(&rsp[3..5]))
}

pub fn read_block_part(
    port: &mut Box<dyn SerialPort>,
    id: u16,
    offset: u16,
    length: u16,
) -> Result<PartialStorageBlock, Box<dyn Error>> {
    let mut data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    BigEndian::write_u16(&mut data[0..2], id);
    BigEndian::write_u16(&mut data[2..4], offset);
    BigEndian::write_u16(&mut data[4..6], length);

    let rsp = send_command(port, CommandType::StorageReadBlockPart, &data)?;

    let swion_result = SwionResult::parse_default(rsp[9]);
    if swion_result.is_error() {
        return Err(SwionError::new(
            "command_read_storage_block_partial".to_string(),
            swion_result,
        )
        .into());
    }

    let block = PartialStorageBlock {
        id: BigEndian::read_u16(&rsp[3..]),
        offset: BigEndian::read_u16(&rsp[5..]),
        length: BigEndian::read_u16(&rsp[7..]),
        data: Vec::from(&rsp[10..]),
    };

    Ok(block)
}
