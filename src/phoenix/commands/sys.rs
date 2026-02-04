use std::error::Error;

use byteorder::{ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::phoenix::{
    commands::{
        send_command, validate_command_response_result_default,
        validate_command_response_result_var1, validate_command_response_type,
    },
    encoding::decode_string,
    types::{CommandType, FeatureFlag},
};

pub fn read_firmware_version(port: &mut Box<dyn SerialPort>) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFirmwareVersion, &[])?;
    validate_command_response_type(&rsp, CommandType::SysReadFirmwareVersion)?;
    Ok(decode_string(&rsp[3..]))
}

pub fn read_serial_number(port: &mut Box<dyn SerialPort>) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadSerialNumber, &[])?;
    Ok(decode_string(&rsp[3..]))
}

pub fn read_feature_flags(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFeatureFlags, &[])?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(&rsp[3..])))
}

pub fn write_feature_flags(
    port: &mut Box<dyn SerialPort>,
    flags: FeatureFlag,
) -> Result<(), Box<dyn Error>> {
    let mut data = [0, 0, 0, 0];
    LittleEndian::write_u32(&mut data, flags.into());
    let rsp = send_command(port, CommandType::SysWriteFeatureFlags, &data)?;
    println!("RSP: {:X?}", rsp);

    validate_command_response_type(&rsp, CommandType::SysWriteFeatureFlags)?;
    validate_command_response_result_var1(&rsp, "command_write_feature_flags")?;
    Ok(())
}

pub fn read_firmware_build_id(
    port: &mut Box<dyn SerialPort>,
) -> Result<String, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysReadFirmwareBuildId, &[])?;
    Ok(decode_string(&rsp[3..]))
}

pub fn start_firmware_update(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::SysStartFirmwareUpdate, &[])?;
    validate_command_response_type(&rsp, CommandType::SysStartFirmwareUpdate)?;
    validate_command_response_result_default(&rsp, "command_start_firmware_update")?;
    Ok(())
}
