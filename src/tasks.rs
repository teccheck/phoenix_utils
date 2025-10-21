use std::error::Error;

use serialport::SerialPort;

use crate::{
    commands::{
        command_read_feature_flags, command_read_firmware_version, command_read_serial_number,
        command_read_storage_block_info, command_read_storage_block_partial,
        command_storage_directory_size,
    },
    types::{DeviceInfo, StorageBlockId, StorageBlockInfo, StorageBlockLength, StorageBlockOffset},
};

pub fn task_print_storage_directory(port: &mut Box<dyn SerialPort>) {
    println!("Reading Storage directory. This might take a few seconds...");

    match task_read_storage_directory(port) {
        Ok(dir) => {
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
        Err(e) => println!("Error reading device info: {}", e),
    }
}

pub fn task_read_storage_directory(
    port: &mut Box<dyn SerialPort>,
) -> Result<Vec<StorageBlockInfo>, Box<dyn Error>> {
    let size = command_storage_directory_size(port)?;
    let mut blocks = vec![];

    for i in 0..size {
        let block = command_read_storage_block_info(port, i)?;
        blocks.push(block);
    }

    Ok(blocks)
}

pub fn task_print_storage_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
    offset: StorageBlockOffset,
    length: StorageBlockLength,
) -> Result<(), Box<dyn Error>> {
    let data = task_read_storage_block(port, id, offset, length)?;
    println!("Storage Block ({:X}): {:X?}", id, data);
    Ok(())
}

pub fn task_read_storage_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
    offset: StorageBlockOffset,
    length: StorageBlockLength,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let part_size = 16;
    let mut index = 0;
    let mut data = Vec::new();

    loop {
        let offset = offset + index * part_size;
        if offset >= length {
            break;
        }

        let len = if length - offset < part_size {
            length - offset
        } else {
            part_size
        };

        if len == 0 {
            break;
        }

        let block = command_read_storage_block_partial(port, id, offset, len)?;
        data.extend_from_slice(&block.data);
        index += 1;
    }

    Ok(data)
}

pub fn task_print_device_info(port: &mut Box<dyn SerialPort>) {
    match task_read_device_info(port) {
        Ok(info) => println!("Device info:\n{}", info),
        Err(e) => println!("Error reading device info: {}", e),
    }
}

pub fn task_read_device_info(port: &mut Box<dyn SerialPort>) -> Result<DeviceInfo, Box<dyn Error>> {
    let serial_number = command_read_serial_number(port)?;
    let firmware_version = command_read_firmware_version(port)?;
    let feature_flags = command_read_feature_flags(port)?;

    Ok(DeviceInfo {
        serial_number,
        firmware_version,
        feature_flags,
    })
}
