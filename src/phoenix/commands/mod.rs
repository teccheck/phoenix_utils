pub mod device_reset;
pub mod feature_flags;
pub mod lock_key;
pub mod storage;
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
