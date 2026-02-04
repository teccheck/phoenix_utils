use std::error::Error;

use serialport::SerialPort;

use crate::phoenix::{
    commands::{
        send_command, validate_command_response_result_var1, validate_command_response_type,
    },
    types::CommandType,
};

pub fn key_click(port: &mut Box<dyn SerialPort>) {
    let args = [];
    let rsp = send_command(port, CommandType::ToolsKeyClick, &args);

    println!("{:X?}", rsp);
}

pub fn backlight_test_mode(
    port: &mut Box<dyn SerialPort>,
    mode: u8,
) -> Result<(), Box<dyn Error>> {
    let args = [mode];
    let rsp = send_command(port, CommandType::ToolsBacklightTestMode, &args)?;
    validate_command_response_type(&rsp, CommandType::ToolsBacklightTestMode)?;
    validate_command_response_result_var1(&rsp, "command_backlight_test_mode")?;
    Ok(())
}

pub fn backlight_normal_mode(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsBacklightNormalMode, &[])?;
    validate_command_response_type(&rsp, CommandType::ToolsBacklightNormalMode)?;
    validate_command_response_result_var1(&rsp, "command_backlight_normal_mode")?;
    Ok(())
}

pub fn led_test_mode(port: &mut Box<dyn SerialPort>, mode: u8) -> Result<(), Box<dyn Error>> {
    let args = [mode];
    let rsp = send_command(port, CommandType::ToolsLedTestMode, &args)?;
    validate_command_response_type(&rsp, CommandType::ToolsLedTestMode)?;
    validate_command_response_result_var1(&rsp, "command_led_test_mode")?;
    Ok(())
}

pub fn led_normal_mode(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::ToolsLedNormalMode, &[])?;
    validate_command_response_type(&rsp, CommandType::ToolsLedNormalMode)?;
    validate_command_response_result_var1(&rsp, "command_led_normal_mode")?;
    Ok(())
}
