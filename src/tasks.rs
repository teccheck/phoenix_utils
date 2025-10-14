use std::error::Error;

use serialport::SerialPort;

use crate::{
    commands::{
        StorageBlockInfo, command_read_feature_flags, command_read_firmware_version,
        command_read_serial_number, command_read_storage_block_info,
        command_storage_directory_size,
    },
    types::DeviceInfo,
};

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
