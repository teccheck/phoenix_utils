use std::error::Error;

use serialport::SerialPort;

use crate::phoenix::{
    commands::{
        send_command, check_response_result_simple_inv, check_response_type,
    },
    types::CommandType,
};

pub fn key_click(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsKeyClick, &[])?;
    let rsp = check_response_type(&rsp, CommandType::ToolsKeyClick)?;
    check_response_result_simple_inv(rsp, "tools_key_click")?;
    Ok(())
}

pub fn backlight_test_mode(port: &mut Box<dyn SerialPort>, mode: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsBacklightTestMode, &[mode])?;
    let rsp = check_response_type(&rsp, CommandType::ToolsBacklightTestMode)?;
    check_response_result_simple_inv(rsp, "command_backlight_test_mode")?;
    Ok(())
}

pub fn backlight_normal_mode(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsBacklightNormalMode, &[])?;
    let rsp = check_response_type(&rsp, CommandType::ToolsBacklightNormalMode)?;
    check_response_result_simple_inv(rsp, "command_backlight_normal_mode")?;
    Ok(())
}

pub fn led_test_mode(port: &mut Box<dyn SerialPort>, mode: u8) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsLedTestMode, &[mode])?;
    let rsp = check_response_type(&rsp, CommandType::ToolsLedTestMode)?;
    check_response_result_simple_inv(rsp, "command_led_test_mode")?;
    Ok(())
}

pub fn led_normal_mode(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsLedNormalMode, &[])?;
    let rsp = check_response_type(&rsp, CommandType::ToolsLedNormalMode)?;
    check_response_result_simple_inv(rsp, "command_led_normal_mode")?;
    Ok(())
}
