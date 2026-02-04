pub mod sys;
pub mod tools;

use std::error::Error;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::{
    phoenix::encoding::decode_string,
    phoenix::sci_frame_protocol::{decode_frame, encode_frame},
    phoenix::swion_result::{SwionError, SwionResult},
    phoenix::types::{
        AuthError, CRACapabilities, CRACapabilityFlags, CommandType, FeatureFlag,
        InvalidResponseTypeError, PartialStorageBlock, ResetType, StorageBlockId, StorageBlockInfo,
        StorageBlockPermissions,
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

pub fn debug_command(port: &mut Box<dyn SerialPort>, command_type: u16, data: &[u8]) {
    let result = send_command_raw(
        port,
        (command_type >> 8) as u8,
        (command_type & 0xFF) as u8,
        data,
    );
    match result {
        Ok(data) => {
            println!("Ok: {:X?}", data);
        }
        Err(e) => {
            println!("Err: {:?}", e);
        }
    }

    println!("Done");
}

pub fn validate_command_response_type(
    resp: &[u8],
    type_required: CommandType,
) -> Result<(), InvalidResponseTypeError> {
    let type_actual = ((resp[0] as u16) << 8) + (resp[2] as u16);
    let type_required = type_required as u16;

    if type_actual != type_required {
        return Err(InvalidResponseTypeError::new(type_required, type_actual));
    }

    Ok(())
}

pub fn validate_command_response_result_default(
    resp: &[u8],
    operation_name: &str,
) -> Result<(), SwionError> {
    validate_command_response_result(SwionResult::parse_default(resp[3]), operation_name)
}

pub fn validate_command_response_result_var1(
    resp: &[u8],
    operation_name: &str,
) -> Result<(), SwionError> {
    validate_command_response_result(SwionResult::parse_var1(resp[3]), operation_name)
}

pub fn validate_command_response_result<'a>(
    result: SwionResult,
    operation_name: &str,
) -> Result<(), SwionError> {
    if result.is_error() {
        return Err(SwionError::new(operation_name.to_string(), result));
    }

    Ok(())
}

pub fn device_reset_reboot(
    port: &mut Box<dyn SerialPort>,
    reset_type: ResetType,
) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetReboot, &[reset_type as u8])?;
    Ok(())
}

pub fn device_reset_shutdown(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetShutdown, &[])?;
    Ok(())
}

pub fn device_reset_startup(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetStartup, &[])?;
    Ok(())
}

pub fn storage_delete_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
) -> Result<(), Box<dyn Error>> {
    let mut data = [2, 0_u8];
    BigEndian::write_u16(&mut data, id);
    let rsp = send_command(port, CommandType::StorageDeleteBlock, &data)?;
    println!("rsp: {:x?}", rsp);

    Ok(())
}

pub fn storage_read_block_info(
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

pub fn storage_read_dir_size(
    port: &mut Box<dyn SerialPort>,
) -> Result<u16, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::StorageReadDirSize, &[])?;
    Ok(BigEndian::read_u16(&rsp[3..5]))
}

pub fn storage_read_block_part(
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

pub fn feature_flags_read_unique_id(port: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadUniqueId, &[])?;
    Ok(rsp[3..].to_vec())
}

pub fn feature_flags_read_enabled(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadEnabled, &[])?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(&rsp[3..])))
}

pub fn feature_flags_read_supported(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadSupported, &[])?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(&rsp[3..])))
}

pub fn cra_capability_read(
    port: &mut Box<dyn SerialPort>,
) -> Result<CRACapabilities, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::CRACapabilityRead, &[])?;

    let swion_result = SwionResult::parse_default(rsp[3]);
    if swion_result.is_error() {
        return Err(SwionError::new("command_cra_cap_read".to_string(), swion_result).into());
    }

    Ok(CRACapabilities {
        flags: CRACapabilityFlags::from(BigEndian::read_u16(&rsp[4..])),
        payload_request: BigEndian::read_u16(&rsp[6..]),
        payload_response: BigEndian::read_u16(&rsp[8..]),
    })
}

pub fn lock_key_read_and_auth(
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
        }
        .into());
    }

    Ok(())
}

pub fn cra_lock_key_write(
    port: &mut Box<dyn SerialPort>,
    key: &[u8],
    enhanced_protection: bool,
) -> Result<(), Box<dyn Error>> {
    let ep = match enhanced_protection {
        true => 1,
        false => 0,
    };

    let mut args: Vec<u8> = Vec::new();
    args.extend_from_slice(key);
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
        }
        .into());
    }

    Ok(())
}

pub fn key_press(port: &mut Box<dyn SerialPort>, key: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::KeyPress, &[key])?;
    validate_command_response_type(&rsp, CommandType::KeyPress)?;
    validate_command_response_result_var1(&rsp, "command_key_press")?;
    Ok(())
}

pub fn key_release(port: &mut Box<dyn SerialPort>, key: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::KeyRelease, &[key])?;
    validate_command_response_type(&rsp, CommandType::KeyRelease)?;
    validate_command_response_result_var1(&rsp, "command_key_release")?;
    Ok(())
}

pub fn display_test_mode(
    port: &mut Box<dyn SerialPort>,
    mode: u8,
) -> Result<(), Box<dyn Error>> {
    let args = [mode];
    let rsp = send_command(port, CommandType::DisplayTestMode, &args)?;
    validate_command_response_type(&rsp, CommandType::DisplayTestMode)?;
    validate_command_response_result_var1(&rsp, "command_display_test_mode")?;
    Ok(())
}
