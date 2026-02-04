use std::error::Error;

use serialport::SerialPort;

use crate::phoenix::{
    commands::send_command,
    types::{CommandType, ResetType},
};

pub fn reboot(
    port: &mut Box<dyn SerialPort>,
    reset_type: ResetType,
) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetReboot, &[reset_type as u8])?;
    Ok(())
}

pub fn shutdown(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetShutdown, &[])?;
    Ok(())
}

pub fn startup(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let _ = send_command(port, CommandType::DeviceResetStartup, &[])?;
    Ok(())
}
