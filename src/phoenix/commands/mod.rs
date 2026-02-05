pub mod boot;
pub mod feature_flags;
pub mod lock_key;
pub mod storage;
pub mod sys;
pub mod time;
pub mod tools;

use std::error::Error;

use serialport::SerialPort;

use crate::phoenix::{
    sci_frame_protocol::{decode_frame, encode_frame, read_until_end},
    swion_result::{SwionError, SwionResult},
    types::{CommandType, InvalidResponseTypeError},
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

    let frame = read_until_end(port)?;
    let rsp = decode_frame(&frame)?;

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

pub fn check_response_type(
    resp: &[u8],
    type_required: CommandType,
) -> Result<&[u8], InvalidResponseTypeError> {
    let type_actual = ((resp[0] as u16) << 8) + (resp[2] as u16);
    let type_required = type_required as u16;

    if type_actual != type_required {
        return Err(InvalidResponseTypeError::new(type_required, type_actual));
    }

    Ok(&resp[3..])
}

pub fn check_response_result_default<'a>(
    resp: &'a [u8],
    operation_name: &str,
) -> Result<&'a [u8], SwionError> {
    check_response_result(resp, SwionResult::parse_default, operation_name)
}

pub fn check_response_result_simple_inv<'a>(
    resp: &'a [u8],
    operation_name: &str,
) -> Result<&'a [u8], SwionError> {
    check_response_result(resp, SwionResult::parse_simple_inv, operation_name)
}

fn check_response_result<'a, F: Fn(u8) -> SwionResult>(
    resp: &'a [u8],
    parser: F,
    operation_name: &str,
) -> Result<&'a [u8], SwionError> {
    let result = parser(resp[0]);
    if result.is_error() {
        return Err(SwionError::new(operation_name.to_string(), result));
    }
    Ok(&resp[1..])
}

pub fn key_press(port: &mut Box<dyn SerialPort>, key: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::KeyPress, &[key])?;
    check_response_type(&rsp, CommandType::KeyPress)?;
    check_response_result_simple_inv(&rsp, "command_key_press")?;
    Ok(())
}

pub fn key_release(port: &mut Box<dyn SerialPort>, key: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::KeyRelease, &[key])?;
    check_response_type(&rsp, CommandType::KeyRelease)?;
    check_response_result_simple_inv(&rsp, "command_key_release")?;
    Ok(())
}

pub fn display_test_mode(port: &mut Box<dyn SerialPort>, mode: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::DisplayTestMode, &[mode])?;
    check_response_type(&rsp, CommandType::DisplayTestMode)?;
    Ok(())
}
