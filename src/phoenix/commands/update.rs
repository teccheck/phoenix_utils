use std::error::Error;

use byteorder::{ByteOrder, LittleEndian};
use serialport::SerialPort;

use crate::phoenix::{
    swion_result::{SwionError, SwionResult},
    types::{InvalidResponseData, InvalidResponseTypeError, UpdateFrameId},
    update_frame_protocol,
};

pub fn debug_command(
    port: &mut Box<dyn SerialPort>,
    id: UpdateFrameId,
    data: &[u8],
) -> Result<(), Box<dyn Error>> {
    let response = send_command(port, id, data)?;
    println!("Result: {:X?}", response);
    Ok(())
}

/// Exits update mode. Will display "Restoring configuration". Can be reset by normal sci command
pub fn reset(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    send_command(port, UpdateFrameId::Reset, &[])?;
    Ok(())
}

/// Checks the compatability id. If correct, unlocks further commands like erease, program and crc
pub fn check(
    port: &mut Box<dyn SerialPort>,
    compatability_id: &[u8],
) -> Result<bool, Box<dyn Error>> {
    send_command(port, UpdateFrameId::Check, compatability_id)?;
    Ok(true)
}

/// WARNING: Danger Zone!
/// Erases the entire firmware rom
pub fn erase(port: &mut Box<dyn SerialPort>) -> Result<bool, Box<dyn Error>> {
    send_command(port, UpdateFrameId::Erase, &[])?;
    Ok(true)
}

/// WARNING: Danger Zone!
/// Writes firm_data into the devices rom starting at address [start]
pub fn program(
    port: &mut Box<dyn SerialPort>,
    start: u32,
    firm_data: &[u8],
) -> Result<bool, Box<dyn Error>> {
    let mut args = Vec::new();
    LittleEndian::write_u24(&mut args, start);
    args.extend_from_slice(&firm_data);

    send_command(port, UpdateFrameId::Program, &args)?;
    Ok(true)
}

/// Calculates the crc16 checksum from start to end
pub fn crc(port: &mut Box<dyn SerialPort>, start: u32, end: u32) -> Result<u16, Box<dyn Error>> {
    let mut args = [0_u8; 6];
    LittleEndian::write_u24(&mut args, start);
    LittleEndian::write_u24(&mut args[3..], end);

    let data = send_command(port, UpdateFrameId::Crc, &args)?;
    Ok(LittleEndian::read_u16(&data[2..]))
}

pub fn dump(port: &mut Box<dyn SerialPort>, arg: u8) -> Result<Vec<u8>, Box<dyn Error>> {
    send_command(port, UpdateFrameId::Dump, &[arg])
}

fn send_command(
    port: &mut Box<dyn SerialPort>,
    frame_id: UpdateFrameId,
    data: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let frame = update_frame_protocol::encode_frame(frame_id as u8, data);
    port.write_all(&frame)?;

    let mut buffer = [0_u8; 64];
    let size = port.read(&mut buffer)?;

    //println!("Resp buff: {:X?}", &buffer[0..size]);

    // For the crc command, there are two responses for whatever reason.
    // We only care for the second, as it has the actual checksum.
    // This is not quite enough. Sometimes it takes a bit longer and the second read is not successful
    // TODO: Read [count] number of bytes
    if size > (buffer[1] + 3).into() {
        let buffer = &buffer[((buffer[1] + 3) as usize)..];
        check_response_ack(frame_id, &buffer[0..size])?;
        return Ok(buffer[2..size - 1].to_vec());
    }

    check_response_ack(frame_id, &buffer[0..size])?;
    Ok(buffer[2..size - 1].to_vec())
}

fn check_response_ack(frame_id: UpdateFrameId, data: &[u8]) -> Result<(), Box<dyn Error>> {
    if data.len() < 4 {
        return Err(InvalidResponseData {}.into());
    }

    if data[0] != UpdateFrameId::Ack as u8 {
        return Err(InvalidResponseData {}.into());
    }

    if data[2] != frame_id as u8 {
        return Err(InvalidResponseTypeError::new(frame_id as u16, data[2] as u16).into());
    }

    if data[3] != 0x00 {
        let result = SwionResult::parse_update_frame(data[3]);
        return Err(SwionError::new("Error in update frame".to_string(), result).into());
    }

    Ok(())
}
