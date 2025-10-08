use std::error::Error;

use serialport::SerialPort;

use crate::commands::{command_storage_block_info, command_storage_directory_size, StorageBlockInfo};

pub fn task_read_storage_directory(port: &mut Box<dyn SerialPort>) -> Result<Vec<StorageBlockInfo>, Box<dyn Error>> {
    let size = command_storage_directory_size(port)?;
    let mut blocks = vec![];

    for i in 0..size {
        let block = command_storage_block_info(port, i)?;
        blocks.push(block);
    }

    Ok(blocks)
}
