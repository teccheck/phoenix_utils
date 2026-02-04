use std::error::Error;

use byteorder::{ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::phoenix::{
    commands::send_command,
    types::{CommandType, FeatureFlag},
};

pub fn read_unique_id(
    port: &mut Box<dyn SerialPort>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadUniqueId, &[])?;
    Ok(rsp[3..].to_vec())
}

pub fn read_enabled(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadEnabled, &[])?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(&rsp[3..])))
}

pub fn read_supported(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::FeatureFlagsReadSupported, &[])?;
    Ok(FeatureFlag::from(LittleEndian::read_u32(&rsp[3..])))
}
