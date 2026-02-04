use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::phoenix::{
    commands::send_command,
    swion_result::{SwionError, SwionResult},
    types::{AuthError, CRACapabilities, CRACapabilityFlags, CommandType},
};

pub fn cra_capability_read(
    port: &mut Box<dyn SerialPort>,
) -> Result<CRACapabilities, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::CRACapabilityRead, &[])?;

    let swion_result = SwionResult::parse_default(rsp[3]);
    if swion_result.is_error() {
        return Err(SwionError::new("command_cra_cap_read".to_string(), swion_result).into());
    }

    Ok(CRACapabilities {
        flags: CRACapabilityFlags::from(BigEndian::read_u16(&rsp[4..])),
        payload_request: BigEndian::read_u16(&rsp[6..]),
        payload_response: BigEndian::read_u16(&rsp[8..]),
    })
}

pub fn read_and_auth(port: &mut Box<dyn SerialPort>, key: &[u8]) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::LockKeyReadAndAuth, key)?;

    let swion_result = SwionResult::parse_auth(rsp[3]);
    if swion_result.is_error() {
        return Err(AuthError {
            result: swion_result,
            remaining_attempts: rsp[4],
            locked_until_day: rsp[5],
            locked_until_month: rsp[6],
            locked_until_year: BigEndian::read_u16(&rsp[7..]),
            enhanced_protection: rsp[9],
        }
        .into());
    }

    Ok(())
}

pub fn cra_lock_key_write(
    port: &mut Box<dyn SerialPort>,
    key: &[u8],
    enhanced_protection: bool,
) -> Result<(), Box<dyn Error>> {
    let ep = match enhanced_protection {
        true => 1,
        false => 0,
    };

    let mut args: Vec<u8> = Vec::new();
    args.extend_from_slice(key);
    args.extend([ep]);

    let rsp = send_command(port, CommandType::CRALockKeyWrite, &args)?;

    let swion_result = SwionResult::parse_auth(rsp[3]);
    if swion_result.is_error() {
        return Err(AuthError {
            result: swion_result,
            remaining_attempts: rsp[4],
            locked_until_day: rsp[5],
            locked_until_month: rsp[6],
            locked_until_year: BigEndian::read_u16(&rsp[7..]),
            enhanced_protection: rsp[9],
        }
        .into());
    }

    Ok(())
}
