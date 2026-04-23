use std::error::Error;

use byteorder::{ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::phoenix::{
    commands::{check_response_result_simple_inv, check_response_type, send_command},
    encoding::decode_string,
    types::{CommandType, FeatureFlag},
};

pub fn read_firmware_version(port: &mut Box<dyn SerialPort>) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFirmwareVersion, &[])?;
    let rsp = check_response_type(&rsp, CommandType::SysReadFirmwareVersion)?;
    Ok(decode_string(rsp))
}

pub fn read_serial_number(port: &mut Box<dyn SerialPort>) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadSerialNumber, &[])?;
    let rsp = check_response_type(&rsp, CommandType::SysReadSerialNumber)?;
    Ok(decode_string(rsp))
}

pub fn read_feature_flags(port: &mut Box<dyn SerialPort>) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFeatureFlags, &[])?;
    let rsp = check_response_type(&rsp, CommandType::SysReadFeatureFlags)?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(rsp)))
}

pub fn write_feature_flags(
    port: &mut Box<dyn SerialPort>,
    flags: FeatureFlag,
) -> Result<(), Box<dyn Error>> {
    let mut data = [0_u8; 4];
    LittleEndian::write_u32(&mut data, flags.into());
    let rsp = send_command(port, CommandType::SysWriteFeatureFlags, &data)?;
    let rsp = check_response_type(&rsp, CommandType::SysWriteFeatureFlags)?;
    check_response_result_simple_inv(rsp, "command_write_feature_flags")?;
    Ok(())
}

pub fn read_firmware_build_id(port: &mut Box<dyn SerialPort>) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFirmwareBuildId, &[])?;
    let rsp = check_response_type(&rsp, CommandType::SysReadFirmwareBuildId)?;
    Ok(decode_string(rsp))
}

pub fn start_firmware_update(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysStartFirmwareUpdate, &[])?;
    let rsp = check_response_type(&rsp, CommandType::SysStartFirmwareUpdate)?;
    check_response_result_simple_inv(rsp, "command_start_firmware_update")?;
    Ok(())
}
