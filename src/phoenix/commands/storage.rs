use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::phoenix::{
    commands::{check_response_result_default, check_response_type, send_command},
    swion_result::{SwionError, SwionResult},
    types::{
        CommandType, PartialStorageBlock, ReadStorageBlock, StorageBlockId, StorageBlockInfo,
        StorageBlockLength, StorageBlockPermissions, StorageBlockVersion,
    },
};

const CRC16: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_KERMIT);

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
    let rsp = check_response_type(&rsp, CommandType::StorageReadBlockInfo)?;

    Ok(StorageBlockInfo {
        id: BigEndian::read_u16(&rsp[0..]),
        length: BigEndian::read_u16(&rsp[2..]),
        version: rsp[4],
        permissions: StorageBlockPermissions::from(rsp[5]),
    })
}

pub fn read_dir_size(port: &mut Box<dyn SerialPort>) -> Result<u16, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::StorageReadDirSize, &[])?;
    let rsp = check_response_type(&rsp, CommandType::StorageReadDirSize)?;
    Ok(BigEndian::read_u16(rsp))
}

pub fn read_block_part(
    port: &mut Box<dyn SerialPort>,
    id: u16,
    offset: u16,
    length: u16,
) -> Result<PartialStorageBlock, Box<dyn Error>> {
    let mut data = [6, 0_u8];
    BigEndian::write_u16(&mut data[0..2], id);
    BigEndian::write_u16(&mut data[2..4], offset);
    BigEndian::write_u16(&mut data[4..6], length);

    let rsp = send_command(port, CommandType::StorageReadBlockPart, &data)?;
    let rsp = check_response_type(&rsp, CommandType::StorageReadBlockPart)?;

    let swion_result = SwionResult::parse_default(rsp[6]);
    if swion_result.is_error() {
        return Err(SwionError::new(
            "command_read_storage_block_partial".to_string(),
            swion_result,
        )
        .into());
    }

    let block = PartialStorageBlock {
        id: BigEndian::read_u16(&rsp[0..]),
        offset: BigEndian::read_u16(&rsp[2..]),
        length: BigEndian::read_u16(&rsp[4..]),
        data: Vec::from(&rsp[7..]),
    };

    Ok(block)
}

// TODO: Checksum for ext commands
pub fn ext_nvm_read_read_dir(
    port: &mut Box<dyn SerialPort>,
) -> Result<Vec<StorageBlockInfo>, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::StorageExtNvmReadDir, &[])?;
    let rsp = check_response_type(&rsp, CommandType::StorageExtNvmReadDir)?;

    let mut blocks = Vec::new();

    for i in (0..rsp.len()).step_by(7) {
        let block_data = &rsp[i..];
        if block_data.len() < 7 {
            break;
        }

        let id: StorageBlockId = BigEndian::read_u16(&block_data[0..]);
        let length: StorageBlockLength = BigEndian::read_u16(&block_data[2..]);
        let version: StorageBlockVersion = block_data[4];
        let permissions = StorageBlockPermissions::from(block_data[5]);
        let following_blocks = block_data[6] as u16 + 1;

        for block_id_offset in 0..following_blocks {
            blocks.push(StorageBlockInfo {
                id: id + block_id_offset,
                length,
                version,
                permissions,
            });
        }
    }

    Ok(blocks)
}

pub fn read_status(port: &mut Box<dyn SerialPort>) -> Result<Option<u16>, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::StorageReadStatus, &[])?;
    let rsp = check_response_type(&rsp, CommandType::StorageReadStatus)?;
    let rsp = check_response_result_default(rsp, "storage_read_status")?;

    // What does that mean? Is this status only present if result is error?
    if rsp.len() >= 2 {
        let status = BigEndian::read_u16(rsp);
        return Ok(Some(status));
    }

    Ok(None)
}

// Class: ac2
pub fn ext_nvm_read(
    port: &mut Box<dyn SerialPort>,
    read_blocks: &[ReadStorageBlock],
) -> Result<Vec<PartialStorageBlock>, Box<dyn Error>> {
    let mut args = Vec::new();

    for block in read_blocks {
        let mut block_args = [0_u8; 6];
        BigEndian::write_u16(&mut block_args[0..], block.id);
        BigEndian::write_u16(&mut block_args[2..], block.offset);
        BigEndian::write_u16(&mut block_args[4..], block.length);
        args.extend_from_slice(&block_args);
    }

    let rsp = send_command(port, CommandType::StorageExtNvmRead, &args)?;
    let rsp = check_response_type(&rsp, CommandType::StorageExtNvmRead)?;
    println!("rsp: {:x?}", rsp);

    let mut partial_blocks = Vec::new();
    let mut index = 0;

    for block in read_blocks {
        let data = &rsp[index..];
        let id = BigEndian::read_u16(&data[0..]);
        let offset = BigEndian::read_u16(&data[2..]);
        let length = BigEndian::read_u16(&data[4..]);
        let result = SwionResult::parse_default(data[6]);

        let len_usize = length as usize;
        let block_end = 7 + length as usize;
        let data = &data[7..block_end];

        partial_blocks.push(PartialStorageBlock {
            id,
            offset,
            length,
            data: data.to_vec(),
        });

        index += block_end;
    }

    let checksum_msg = BigEndian::read_u16(&rsp[rsp.len() - 2..]);
    let checksum_calc = CRC16.checksum(&rsp[0..rsp.len() - 2]);

    if checksum_msg != checksum_calc {
        return Err(SwionError::new(
            "storage_ext_nvm_read".to_string(),
            SwionResult::ChecksumMismatch,
        )
        .into());
    }

    Ok(partial_blocks)
}

// Class: ac3
pub fn ext_nvm_write(port: &mut Box<dyn SerialPort>, write_blocks: &[PartialStorageBlock]) -> Result<(), Box<dyn Error>> {
    let mut args = Vec::new();

    for block in write_blocks {
        let mut block_args = [0_u8; 6];
        BigEndian::write_u16(&mut block_args[0..], block.id);
        BigEndian::write_u16(&mut block_args[2..], block.offset);
        BigEndian::write_u16(&mut block_args[4..], block.length);
        args.extend_from_slice(&block_args);
        args.extend_from_slice(&block.data);
    }

    let checksum = CRC16.checksum(&args);
    args.extend_from_slice(&[0_u8; 2]);
    let checksum_index = args.len() - 2;
    BigEndian::write_u16(&mut args[checksum_index..], checksum);

    println!("args: {:x?}", args);

    let rsp = send_command(port, CommandType::StorageExtNvmWrite, &args)?;    
    let rsp = check_response_type(&rsp, CommandType::StorageExtNvmWrite)?;
    println!("rsp: {:x?}", rsp);

    let mut index = 0;
    for block in write_blocks {
        let data = &rsp[index..];
        let id = BigEndian::read_u16(&data[0..]);
        let offset = BigEndian::read_u16(&data[2..]);
        let length = BigEndian::read_u16(&data[4..]);
        let result = SwionResult::parse_default(data[6]);

        if id != block.id || offset != block.offset || length != block.length || result.is_error() {
            return Err(SwionError::new("storage_ext_nvm_write".to_string(), SwionResult::DataInvalid).into());
        }

        index += 7;
    }

    Ok(())
}
