pub fn encode_frame(id: u8, data: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(id);
    frame.push(data.len() as u8);
    frame.extend_from_slice(data);
    frame.push(checksum(&frame));
    frame
}

fn checksum(data: &[u8]) -> u8 {
    !data.iter().fold(0, |a, b: &u8| u8::wrapping_add(a, *b))
}
