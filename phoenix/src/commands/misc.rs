use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::{
    commands::{check_response_result_simple_inv, check_response_type, send_command}, types::CommandType,
};

pub fn send_test_message(
    port: &mut Box<dyn SerialPort>,
    address: u32,
    sub: u8,
    text: &str,
) -> Result<(), Box<dyn Error>> {
    let mut args = [0u8; 17];
    BigEndian::write_u32(&mut args[1..], address);
    args[5] = sub;
    args[6] = 0; // Some argument, but is always 0?
    args[7] = 135;

    let mut vec = Vec::new();
    vec.extend_from_slice(&args);
    vec.extend_from_slice(&text.as_bytes());

    let rsp = send_command(port, CommandType::TestMessageSend, &vec)?;
    check_response_type(&rsp, CommandType::TestMessageSend)?;
    Ok(())
}

// Untested
pub fn send_test_message_flex(
    port: &mut Box<dyn SerialPort>,
    capcode: String,
    text: &str,
) -> Result<(), Box<dyn Error>> {
    let mut vec = Vec::new();
    vec.extend_from_slice(&capcode.as_bytes());
    vec.resize(16, 0);
    vec.extend_from_slice(&text.as_bytes());

    println!("Buffer: {:X?}", &vec);

    let rsp = send_command(port, CommandType::Tools2FlexTestMessage, &vec)?;
    check_response_type(&rsp, CommandType::Tools2FlexTestMessage)?;
    Ok(())
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
