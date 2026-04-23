use std::error::Error;

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

pub fn reset(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    send_command(port, UpdateFrameId::Reset, &[])?;
    Ok(())
}

pub fn check(
    port: &mut Box<dyn SerialPort>,
    compatability_id: &[u8],
) -> Result<bool, Box<dyn Error>> {
    send_command(port, UpdateFrameId::Check, compatability_id)?;
    Ok(true)
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

    check_response_ack(frame_id, &buffer[0..size])?;

    Ok(buffer[2..size - 1].to_vec())
}

fn check_response_ack(frame_id: UpdateFrameId, data: &[u8]) -> Result<(), Box<dyn Error>> {
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
