use std::error::Error;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::{
    phoenix_encoding::decode_string,
    sci_frame_protocol::{decode_frame, encode_frame},
    swion_result::{SwionError, SwionResult},
    types::{
        AuthError, CRACapabilities, CRACapabilityFlags, CommandType, FeatureFlag, PartialStorageBlock, ResetType, StorageBlockId, StorageBlockInfo, StorageBlockPermissions
    },
};

fn send_command(
    port: &mut Box<dyn SerialPort>,
    command_type: CommandType,
    data: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cmd = command_type as u16;
    send_command_raw(port, (cmd >> 8) as u8, (cmd & 0xFF) as u8, data)
}

fn send_command_raw(
    port: &mut Box<dyn SerialPort>,
    command_type: u8,
    command_sub_type: u8,
    data: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let header = [command_type, 0x03, command_sub_type];
    let mut msg = Vec::from(header);
    msg.extend_from_slice(data);

    let frame = encode_frame(&msg);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 64] = [0; 64];
    let size = port.read(&mut read_buf)?;
    let rsp = decode_frame(&read_buf[..size])?;

    Ok(rsp)
}

pub fn debug_command(
    port: &mut Box<dyn SerialPort>,
    command_type: u8,
    command_sub_type: u8,
    data: &[u8],
) {
    println!(
        "Debug command {:X}{:X} ({}:{}):",
        command_type, command_sub_type, command_type, command_sub_type
    );

    println!("Data: {:X?}", data);

    let result = send_command_raw(port, command_type, command_sub_type, data);
    match result {
        Ok(data) => {
            println!("Ok: {:X?}", data);
        },
        Err(e) => {
            println!("Err: {:?}", e);
        },
    }

    println!("Done");
}

pub fn command_read_firmware_version(
    port: &mut Box<dyn SerialPort>,
) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFirmwareVersion, &[])?;
    Ok(decode_string(&rsp))
}

pub fn command_read_serial_number(
    port: &mut Box<dyn SerialPort>,
) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadSerialNumber, &[])?;
    Ok(decode_string(&rsp))
}

pub fn command_read_feature_flags(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFeatureFlags, &[])?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(&rsp[3..])))
}

pub fn command_reset_device(
    port: &mut Box<dyn SerialPort>,
    reset_type: ResetType,
) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetReboot, &[reset_type as u8])?;
    Ok(())
}

pub fn command_shutdown_device(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetShutdown, &[])?;
    Ok(())
}

pub fn command_bootup_device(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetStartup, &[])?;
    Ok(())
}

pub fn command_delete_storage_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
) -> Result<(), Box<dyn Error>> {
    let mut data = [2, 0 as u8];
    BigEndian::write_u16(&mut data, id);
    let rsp = send_command(port, CommandType::StorageDeleteBlock, &data)?;
    println!("rsp: {:x?}", rsp);

    Ok(())
}

pub fn command_read_storage_block_info(
    port: &mut Box<dyn SerialPort>,
    index: u16,
) -> Result<StorageBlockInfo, Box<dyn Error>> {
    let mut data = [2, 0 as u8];
    BigEndian::write_u16(&mut data, index);
    let rsp = send_command(port, CommandType::StorageReadBlockInfo, &data)?;

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
    let rsp = send_command(port, CommandType::StorageReadDirSize, &[])?;
    Ok(BigEndian::read_u16(&rsp[3..5]))
}

pub fn command_read_storage_block_partial(
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
        return Err(SwionError::new("command_read_storage_block_partial".to_string(), swion_result).into());
    }

    let block = PartialStorageBlock {
        id: BigEndian::read_u16(&rsp[3..]),
        offset: BigEndian::read_u16(&rsp[5..]),
        length: BigEndian::read_u16(&rsp[7..]),
        data: Vec::from(&rsp[10..]),
    };

    Ok(block)
}

// No idea what to send here?
pub fn command_write_feature_flags(
    port: &mut Box<dyn SerialPort>,
    flags: &[u8],
) -> Result<(), Box<dyn Error>> {
    let msg = [0x43, 0x03, 0x00];
    let mut whole: Vec<u8> = Vec::new();
    whole.extend_from_slice(&msg);
    whole.extend_from_slice(flags);

    let frame = encode_frame(&whole);
    port.write_all(&frame)?;

    let mut read_buf: [u8; 256] = [0; 256];
    let size = port.read(&mut read_buf)?;
    println!("Readbuf: {:x?}", read_buf);

    let rsp = decode_frame(&read_buf[..size])?;

    if rsp.len() < 5 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "writing feature flag failed",
        )
        .into());
    }

    Ok(())
}

pub fn command_read_unique_id(port: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadUniqueId, &[])?;
    Ok(rsp[3..].to_vec())
}

pub fn command_read_feature_flags_enabled(
    port: &mut Box<dyn SerialPort>,
) -> Result<u32, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadEnabled, &[])?;
    Ok(LittleEndian::read_u32(&rsp[3..]))
}

pub fn command_read_feature_flags_available(
    port: &mut Box<dyn SerialPort>,
) -> Result<u32, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadSupported, &[])?;
    Ok(LittleEndian::read_u32(&rsp[3..]))
}

pub fn command_cra_cap_read(
    port: &mut Box<dyn SerialPort>,
) -> Result<CRACapabilities, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::CRACapabilityRead, &[])?;

    let swion_result = SwionResult::parse_default(rsp[3]);
    if swion_result.is_error() {
        return Err(SwionError::new("command_cra_cap_read".to_string(), swion_result).into());
    }

    Ok(CRACapabilities {
        flags: CRACapabilityFlags::from(BigEndian::read_u16(&rsp[4..])),
        payloadRequest: BigEndian::read_u16(&rsp[6..]),
        payloadResponse: BigEndian::read_u16(&rsp[8..]),
    })
}

pub fn command_lock_key_auth(
    port: &mut Box<dyn SerialPort>,
    key: &[u8],
) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::LockKeyReadAndAuth, key)?;

    let swion_result = SwionResult::parse_var1(rsp[3]);
    if swion_result.is_error() {
        return Err(AuthError {
            result: swion_result,
            remaining_attempts: rsp[4],
            locked_until_day: rsp[5],
            locked_until_month: rsp[6],
            locked_until_year: BigEndian::read_u16(&rsp[7..]),
            enhanced_protection: rsp[9],
        }.into());
    }

    Ok(())
}

pub fn command_lock_key_write(
    port: &mut Box<dyn SerialPort>,
    key: &[u8],
    enhanced_protection: bool,
) -> Result<(), Box<dyn Error>> {
    let ep = match enhanced_protection {
        true => 1,
        false => 0,
    };

    let mut args: Vec<u8> = Vec::new();
    args.extend_from_slice(&key);
    args.extend([ep]);

    let rsp = send_command(port, CommandType::CRALockKeyWrite, &args)?;

    let swion_result = SwionResult::parse_var1(rsp[3]);
    if swion_result.is_error() {
        return Err(AuthError {
            result: swion_result,
            remaining_attempts: rsp[4],
            locked_until_day: rsp[5],
            locked_until_month: rsp[6],
            locked_until_year: BigEndian::read_u16(&rsp[7..]),
            enhanced_protection: rsp[9],
        }.into());
    }

    Ok(())
}
