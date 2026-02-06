use std::error::Error;

use byteorder::{ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::phoenix::{
    commands::{send_command, check_response_type},
    types::{CommandType, FeatureFlag},
};

pub fn read_unique_id(
    port: &mut Box<dyn SerialPort>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadUniqueId, &[])?;
    let rsp = check_response_type(&rsp, CommandType::FeatureFlagsReadUniqueId)?;
    Ok(rsp.to_vec())
}

pub fn read_enabled(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadEnabled, &[])?;
    let rsp = check_response_type(&rsp, CommandType::FeatureFlagsReadEnabled)?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(rsp)))
}

pub fn read_supported(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadSupported, &[])?;
    let rsp = check_response_type(&rsp, CommandType::FeatureFlagsReadSupported)?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(rsp)))
}
