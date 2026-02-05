use std::time::Duration;

use serialport::{Error, SerialPort};

use crate::phoenix::types::{DeviceType, SerialConfig, SERIAL_CONFIG_1, SERIAL_CONFIG_2};

fn handshake(port: &mut Box<dyn SerialPort>) -> Result<DeviceType, Error> {
    let hello: [u8; 3] = [0x55, 0x7e, 0x55];
    let expected: [u8; 2] = [0x56, 0x56];

    let mut read_buf: [u8; 2] = [0; 2];

    loop {
        port.write_all(&hello)?;
        let size = port.read(&mut read_buf)?;

        if size == 2 && read_buf.starts_with(&expected) {
            println!("Handshake sucessful");
            break;
        }
    }

    let device_tpye_cmd: [u8; 1] = [0x55];
    port.write_all(&device_tpye_cmd)?;
    let _size = port.read(&mut read_buf)?;

    let device_type = DeviceType::parse(read_buf[0]);

    Ok(device_type)
}

fn try_init_connection(
    path: &String,
    config: SerialConfig,
) -> Result<(Box<dyn SerialPort>, DeviceType), Error> {
    let mut serial_port = serialport::new(path, config.baudrate)
        .data_bits(config.databits)
        .parity(config.parity)
        .stop_bits(config.stopbits)
        .timeout(Duration::from_millis(1000))
        .open()?;

    let device_type = handshake(&mut serial_port)?;
    Ok((serial_port, device_type))
}

pub fn init_connection(path: &String) -> Result<(Box<dyn SerialPort>, DeviceType), Error> {
    match try_init_connection(path, SERIAL_CONFIG_1) {
        Ok(tuple) => return Ok(tuple),
        Err(_) => {}
    }

    match try_init_connection(path, SERIAL_CONFIG_2) {
        Ok(tuple) => return Ok(tuple),
        Err(_) => {}
    }

    Err(Error {
        kind: serialport::ErrorKind::Unknown,
        description: "No suitable serial config found".to_string(),
    })
}
