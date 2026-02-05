use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use serialport::SerialPort;

use crate::phoenix::{
    commands::{check_response_result_default, check_response_type, send_command},
    swion_result::SwionResult,
    types::{AuthError, CRACapabilities, CRACapabilityFlags, CommandType},
};

fn check_auth_error(resp: &[u8]) -> Result<(), AuthError> {
    let result = SwionResult::parse_auth(resp[0]);
    if result.is_error() {
        return Err(AuthError {
            result,
            remaining_attempts: resp[1],
            locked_until_day: resp[2],
            locked_until_month: resp[3],
            locked_until_year: BigEndian::read_u16(&resp[4..]),
            enhanced_protection: resp[6],
        });
    }

    Ok(())
}

pub fn read_and_auth(port: &mut Box<dyn SerialPort>, key: &[u8]) -> Result<(), Box<dyn Error>> {
    let rsp = send_command(port, CommandType::LockKeyReadAndAuth, key)?;
    let rsp = check_response_type(&rsp, CommandType::LockKeyReadAndAuth)?;
    check_auth_error(&rsp)?;
    Ok(())
}

pub fn capability_read(port: &mut Box<dyn SerialPort>) -> Result<CRACapabilities, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::CapabilityRead, &[])?;
    let rsp = check_response_type(&rsp, CommandType::CapabilityRead)?;
    let rsp = check_response_result_default(&rsp, "cra_capability_read")?;

    Ok(CRACapabilities {
        flags: CRACapabilityFlags::from(BigEndian::read_u16(&rsp[0..])),
        payload_request: BigEndian::read_u16(&rsp[2..]),
        payload_response: BigEndian::read_u16(&rsp[4..]),
    })
}

pub fn cra_write(
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

    let rsp = send_command(port, CommandType::LockKeyCraWrite, &args)?;
    let rsp = check_response_type(&rsp, CommandType::LockKeyCraWrite)?;
    check_auth_error(&rsp)?;
    Ok(())
}
