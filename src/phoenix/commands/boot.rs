use std::error::Error;

use serialport::SerialPort;

use crate::phoenix::{
    commands::{check_response_type, send_command},
    types::{CommandType, ResetType},
};

pub fn reboot(
    port: &mut Box<dyn SerialPort>,
    reset_type: ResetType,
) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::BootReboot, &[reset_type as u8])?;
    check_response_type(&rsp, CommandType::BootReboot)?;
    Ok(())
}

pub fn shutdown(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::BootShutdown, &[])?;
    check_response_type(&rsp, CommandType::BootShutdown)?;
    Ok(())
}

pub fn startup(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::BootStartup, &[])?;
    check_response_type(&rsp, CommandType::BootStartup)?;
    Ok(())
}
