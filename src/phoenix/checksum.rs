pub const CRC8: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::Algorithm {
    width: 8,
    poly: 0x31,
    init: 0xff,
    refin: true,
    refout: true,
    xorout: 0x00,
    check: 0x00,
    residue: 0x00,
});

pub const CRC16: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_KERMIT);

// TODO: Fix
pub const CRC16_UPDATE: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::Algorithm {
    width: 16,
    poly: 0x8005,
    init: 0xffff,
    refin: false,
    refout: false,
    xorout: 0xFFFF,
    check: 0x00,
    residue: 0x00,
});

pub fn update_checksum(data: &[u8]) -> u16 {
    let mut start_addr = 0;
    let end_addr = data.len() - 1;

    // Combine two bytes into a u16
    let mut result = ((data[start_addr] as u16) << 8) | data[start_addr + 1] as u16;

    start_addr += 2;
    while start_addr <= end_addr {
        let mut tmp = ((data[start_addr] as u16) << 8) | data[start_addr + 1] as u16;
        // num4 = ??

        for _ in 0..16 {
            let flag = (result & 0x8000) != 0;
            result = result << 1;
            if flag {
                result ^= 0x8005;
            }

            let flag = (tmp & 0x8000) != 0;
            tmp = tmp << 1;
            if flag {
                result ^= 1;
            }
        }

        start_addr += 2;
    }

    return !result;
}

fn crc_pair(first: u16, second: u16) -> u16 {
    let mut result = first;
    let mut tmp = second;

    for _ in 0..16 {
        let flag = (result & 0x8000) != 0;
        result <<= 1;
        if flag {
            result ^= 0x8005;
        }

        let flag = (tmp & 0x8000) != 0;
        tmp <<= 1;
        if flag {
            result ^= 1;
        }
    }

    !result
}

pub fn recover_second_word(first: u16, observed_crc: u16) -> Option<u16> {
    for candidate in 0u16..=u16::MAX {
        if crc_pair(first, candidate) == observed_crc {
            return Some(candidate);
        }
    }
    None
}
