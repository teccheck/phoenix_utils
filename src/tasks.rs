use std::{error::Error, num::ParseIntError};

use serialport::SerialPort;
use sha1::{Digest, Sha1};

use crate::{
    commands::{
        command_cra_cap_read, command_lock_key_auth, command_lock_key_write,
        command_read_feature_flags, command_read_firmware_build_id, command_read_firmware_version,
        command_read_serial_number, command_read_storage_block_info,
        command_read_storage_block_partial, command_storage_directory_size,
        command_write_feature_flags, debug_command,
    },
    swion_result::SwionResult,
    types::{
        CRACapabilityFlags, DeviceInfo, FeatureFlag, FeatureFlagNotFoundError, StorageBlockId,
        StorageBlockInfo, StorageBlockLength, StorageBlockOffset,
    },
};

pub fn debug_task(
    port: &mut Box<dyn SerialPort>,
    command_type: u16,
    data: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let args = if let Some(d) = data {
        decode_hex(&d)?
    } else {
        Vec::new()
    };

    debug_command(port, command_type, args.as_slice());
    Ok(())
}

pub fn task_write_feature_flags(
    port: &mut Box<dyn SerialPort>,
    flags: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let new_flags = parse_flags_vec(flags)?;
    println!("Write Feature Flags: [{}]", new_flags);

    command_write_feature_flags(port, new_flags)
}

fn parse_flags_vec(flags: Vec<String>) -> Result<FeatureFlag, Box<dyn Error>> {
    let new_flags: Result<Vec<FeatureFlag>, FeatureFlagNotFoundError> =
        flags.iter().map(find_feature_flag_by_string).collect();

    let new_flags = new_flags?
        .into_iter()
        .reduce(FeatureFlag::or)
        .unwrap_or_else(|| FeatureFlag::none());

    Ok(new_flags)
}

fn find_feature_flag_by_string(flag: &String) -> Result<FeatureFlag, FeatureFlagNotFoundError> {
    FeatureFlag::flags()
        .find(|(n, _)| n.eq(flag))
        .map(|(_, f)| *f)
        .ok_or(FeatureFlagNotFoundError {
            flag_name: flag.to_string(),
        })
}

pub fn task_print_storage_directory(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    println!("Reading Storage directory. This might take a few seconds...");
    let dir = task_read_storage_directory(port)?;

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

    Ok(())
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
        Ok(info) => println!("{}", info),
        Err(e) => println!("Error reading device info: {}", e),
    }
}

pub fn task_read_device_info(port: &mut Box<dyn SerialPort>) -> Result<DeviceInfo, Box<dyn Error>> {
    let serial_number = command_read_serial_number(port)?;
    let firmware_version = command_read_firmware_version(port)?;
    let firmware_build_id = command_read_firmware_build_id(port)?;
    let feature_flags = command_read_feature_flags(port)?;

    Ok(DeviceInfo {
        serial_number,
        firmware_version,
        firmware_build_id,
        feature_flags,
    })
}

pub fn task_print_cra_capabilities(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let capabilities = command_cra_cap_read(port)?;
    println!("Capabilities:\n{}", capabilities);
    Ok(())
}

pub fn task_try_authenticate(
    port: &mut Box<dyn SerialPort>,
    password: Option<String>,
    hash_string: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let caps = command_cra_cap_read(port)?;

    let needs_auth = caps.flags.contains(CRACapabilityFlags::LockKeyCommands)
        || caps.flags.contains(CRACapabilityFlags::LockKeyCRACommands);

    if !needs_auth {
        println!("Authentication not needed");
        return Ok(());
    }

    println!("Trying to authenticate...");
    if let Some(hash) = hash_string {
        task_auth_hash_string(port, &hash)
    } else if let Some(password) = password {
        task_auth_password(port, password)
    } else {
        task_auth_hash_string(port, "0000000000000000000000000000000000000000")
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn task_auth_hash_string(
    port: &mut Box<dyn SerialPort>,
    hash_string: &str,
) -> Result<(), Box<dyn Error>> {
    let hash = decode_hex(hash_string)?;
    command_lock_key_auth(port, &hash)
}

pub fn task_auth_password(
    port: &mut Box<dyn SerialPort>,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut hasher = Sha1::new();
    hasher.update(password);
    let hash = hasher.finalize();
    println!("Password Hash: {:X}", hash);

    command_lock_key_auth(port, &hash)
}

pub fn task_reset_password(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let hash: [u8; 20] = [0; 20];
    command_lock_key_write(port, &hash, false)
}

pub fn task_set_password(
    port: &mut Box<dyn SerialPort>,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut hasher = Sha1::new();
    hasher.update(password);
    let hash = hasher.finalize();
    command_lock_key_write(port, &hash, false)
}
