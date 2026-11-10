use std::time::Duration;

use serialport::{Error, SerialPort};

use crate::types::{DeviceType, SERIAL_CONFIG_1, SERIAL_CONFIG_2, SerialConfig};

fn handshake(port: &mut Box<dyn SerialPort>) -> Result<DeviceType, Error> {
    let hello: [u8; 3] = [0x55, 0x7e, 0x55];
    let expected: [u8; 2] = [0x56, 0x56];

    let mut read_buf: [u8; 2] = [0; 2];

    for _ in [0..10] {
        // The clear seems to be important on some systems (perhaps because of the UART adapter?)
        // It makes the handshake much more stable
        port.clear(serialport::ClearBuffer::All)?;
        port.write_all(&hello)?;
        port.read_exact(&mut read_buf)?;

        if read_buf.starts_with(&expected) {
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

pub fn try_init_connection(
    path: &str,
    config: SerialConfig,
    do_handshake: bool,
) -> Result<(Box<dyn SerialPort>, DeviceType), Error> {
    let mut serial_port = serialport::new(path, config.baudrate)
        .data_bits(config.databits)
        .parity(config.parity)
        .stop_bits(config.stopbits)
        .timeout(Duration::from_millis(500))
        .open()?;

    if do_handshake {
        let device_type = handshake(&mut serial_port)?;
        return Ok((serial_port, device_type));
    }

    Ok((serial_port, DeviceType::None))
}

pub fn init_connection(path: &str) -> Result<(Box<dyn SerialPort>, DeviceType), Error> {
    if let Ok(tuple) = try_init_connection(path, SERIAL_CONFIG_1, true) {
        return Ok(tuple);
    }

    if let Ok(tuple) = try_init_connection(path, SERIAL_CONFIG_2, true) {
        return Ok(tuple);
    }

    Err(Error {
        kind: serialport::ErrorKind::Unknown,
        description: "No suitable serial config found".to_string(),
    })
}
