// TODO: Compression and encryption?

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

fn stuff_bytes(data: &[u8]) -> Vec<u8> {
    let mut new_data: Vec<u8> = Vec::new();

    for byte in data {
        if *byte == FRAME_MARKER || *byte == FRAME_ESCAPE {
            new_data.push(FRAME_ESCAPE);
            new_data.push(*byte ^ FRAME_ESCAPE_XOR);
        } else {
            new_data.push(*byte);
        }
    }

    return new_data;
}

pub fn encode_frame(data: &[u8]) -> Vec<u8> {
    let mut stuffed_bytes = stuff_bytes(data);
    let mut stuffed_checksum = stuff_bytes(&[CRC8.checksum(data)]);

    let mut frame = Vec::new();
    frame.push(FRAME_MARKER);
    frame.append(&mut stuffed_bytes);
    frame.append(&mut stuffed_checksum);
    frame.push(FRAME_MARKER);
    return frame;
}
