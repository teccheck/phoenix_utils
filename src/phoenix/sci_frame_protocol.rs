use std::io::Error;

const CRC8: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::Algorithm {
    width: 8,
    poly: 0x31,
    init: 0xff,
    refin: true,
    refout: true,
    xorout: 0x00,
    check: 0x00,
    residue: 0x00,
});
const FRAME_MARKER: u8 = 0x7E;
const FRAME_ESCAPE: u8 = 0x7D;
const FRAME_ESCAPE_XOR: u8 = 0x20;

fn stuff(data: &[u8]) -> Vec<u8> {
    let mut new_data: Vec<u8> = Vec::new();

    for byte in data {
        if *byte == FRAME_MARKER || *byte == FRAME_ESCAPE {
            new_data.push(FRAME_ESCAPE);
            new_data.push(*byte ^ FRAME_ESCAPE_XOR);
        } else {
            new_data.push(*byte);
        }
    }

    new_data
}

fn unstuff(data: &[u8]) -> Vec<u8> {
    let mut new_data: Vec<u8> = Vec::new();
    let mut escape = false;

    for byte in data {
        if *byte == FRAME_ESCAPE {
            escape = true;
        } else if escape {
            new_data.push(*byte ^ FRAME_ESCAPE_XOR);
            escape = false;
        } else {
            new_data.push(*byte);
        }
    }

    new_data
}

pub fn encode_frame(data: &[u8]) -> Vec<u8> {
    let mut stuffed_bytes = stuff(data);
    let mut stuffed_checksum = stuff(&[CRC8.checksum(data)]);

    let mut frame = Vec::new();
    frame.push(FRAME_MARKER);
    frame.append(&mut stuffed_bytes);
    frame.append(&mut stuffed_checksum);
    frame.push(FRAME_MARKER);

    frame
}

pub fn decode_frame(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data[0] != FRAME_MARKER {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "sci start frame marker missing",
        ));
    }

    if !matches!(data.last(), Some(&FRAME_MARKER)) {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "sci end frame marker missing",
        ));
    }

    let unstuffed_data = unstuff(&data[1..data.len() - 1]);

    if unstuffed_data
        .last()
        .is_some_and(|b| *b != CRC8.checksum(&unstuffed_data[..unstuffed_data.len() - 1]))
    {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "sci frame checksum is incorrect",
        ));
    }

    Ok(unstuffed_data[..&unstuffed_data.len() - 1].to_vec())
}
