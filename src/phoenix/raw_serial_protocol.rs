use serialport::{Error, SerialPort};

use crate::phoenix::types::DeviceType;

pub fn handshake(port: &mut Box<dyn SerialPort>) -> Result<DeviceType, Error> {
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

    let device_type = match read_buf[0] {
        0x55 => DeviceType::B,
        0x56 => DeviceType::DE10A,
        0x57 => DeviceType::D,
        0x58 => DeviceType::E,
        0x59 => DeviceType::F,
        _ => DeviceType::A,
    };

    Ok(device_type)
}
