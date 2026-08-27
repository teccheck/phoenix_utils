use std::{error::Error, fs::File, io::Write, path::Path, thread::sleep, time::Duration};

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serialport::SerialPort;
use sha1::{Digest, Sha1};

use crate::{
    checksum::{CRC16_UPDATE, recover_second_word},
    commands,
    types::{
        CRACapabilityFlags, DeviceInfo, FeatureFlag, PartialStorageBlock, StorageBlockId,
        StorageBlockInfo, StorageBlockLength, StorageBlockOffset,
    },
};

pub fn dump_storage(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    println!("Reading Storage directory. This might take a few seconds...");
    let dir = read_storage_directory(port)?;

    for block in dir {
        let data = read_storage_block(port, block.id, 0, block.length);

        println!(
            "| {:>4x} | {:>7} | {:>6} | {:>5} | {:X?} |",
            block.id,
            block.version,
            block.length,
            block.permissions.flag_string(),
            data,
        );

        if let Ok(d) = data {
            dump_storage_block_to_file(&block, &d)
        }
    }

    Ok(())
}

pub fn dump_storage_block_to_file(block: &StorageBlockInfo, data: &[u8]) {
    let pathname = format!("blocks/block_{:0>4x}", block.id);
    let path = Path::new(&pathname);
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let mut out_data: Vec<u8> = Vec::new();
    let mut header = [0; 6];
    LittleEndian::write_u16(&mut header, block.id);
    header[2] = block.version;
    LittleEndian::write_u16(&mut header[3..], block.length);
    header[5] = block.permissions.bits();

    out_data.extend_from_slice(&header);
    out_data.extend_from_slice(data);

    match file.write_all(&out_data) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

pub fn read_storage_directory(
    port: &mut Box<dyn SerialPort>,
) -> Result<Vec<StorageBlockInfo>, Box<dyn Error>> {
    if check_has_cra_capabilities(port, CRACapabilityFlags::ExtendedNVMCommands)? {
        return commands::storage::ext_nvm_read_read_dir(port);
    }

    let size = commands::storage::read_dir_size(port)?;
    let mut blocks = vec![];
    println!("Storage dir has size {size}");

    for i in 0..size {
        print!("\rREAD DIR: {i} / {size}");
        let block = commands::storage::read_block_info(port, i)?;
        blocks.push(block);
    }

    Ok(blocks)
}

pub fn read_storage_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
    offset: StorageBlockOffset,
    length: StorageBlockLength,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let part_size = 16;
    let mut index = 0;
    let mut data = Vec::new();

    loop {
        let offset = offset + index * part_size;
        if offset >= length {
            break;
        }

        let len = if length - offset < part_size {
            length - offset
        } else {
            part_size
        };

        if len == 0 {
            break;
        }

        let block = commands::storage::read_block_part(port, id, offset, len)?;
        data.extend_from_slice(&block.data);
        index += 1;
    }

    Ok(data)
}

pub fn write_storage_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
    offset: StorageBlockOffset,
    length: StorageBlockLength,
    data: &[u8],
) -> Result<(), Box<dyn Error>> {
    commands::storage::ext_nvm_write(
        port,
        &[PartialStorageBlock {
            id,
            offset,
            length,
            data: data.to_vec(),
        }],
    )
}

pub fn read_device_info(port: &mut Box<dyn SerialPort>) -> Result<DeviceInfo, Box<dyn Error>> {
    let serial_number = commands::sys::read_serial_number(port)?;
    let firmware_version = commands::sys::read_firmware_version(port)?;
    let firmware_build_id = commands::sys::read_firmware_build_id(port)?;
    let feature_flags = commands::sys::read_feature_flags(port)?;

    Ok(DeviceInfo {
        serial_number,
        firmware_version,
        firmware_build_id,
        feature_flags,
    })
}

pub fn try_authenticate(
    port: &mut Box<dyn SerialPort>,
    password: Option<String>,
    hash: Option<Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    let needs_auth = check_has_cra_capabilities(
        port,
        CRACapabilityFlags::LockKeyCRACommands.or(CRACapabilityFlags::LockKeyCommands),
    )?;

    if !needs_auth {
        return Ok(());
    }

    if let Some(hash) = hash {
        auth_hash_string(port, &hash)
    } else if let Some(password) = password {
        auth_password(port, password)
    } else {
        auth_hash_string(port, &[0_u8, 20])
    }
}

pub fn auth_hash_string(port: &mut Box<dyn SerialPort>, hash: &[u8]) -> Result<(), Box<dyn Error>> {
    commands::lock_key::read_and_auth(port, &hash)
}

pub fn auth_password(
    port: &mut Box<dyn SerialPort>,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut hasher = Sha1::new();
    hasher.update(password);
    let hash = hasher.finalize();
    commands::lock_key::read_and_auth(port, &hash)
}

pub fn reset_password(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    commands::lock_key::cra_write(port, &[0_u8; 20], false)
}

pub fn set_password(
    port: &mut Box<dyn SerialPort>,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut hasher = Sha1::new();
    hasher.update(password);
    let hash = hasher.finalize();
    commands::lock_key::cra_write(port, &hash, false)
}

pub fn feature_flags_read_enabled(
    port: &mut Box<dyn SerialPort>,
) -> Result<FeatureFlag, Box<dyn Error>> {
    if check_has_cra_capabilities(port, CRACapabilityFlags::FeatureFlagCommands)? {
        commands::sys::read_feature_flags(port)
    } else {
        commands::feature_flags::read_enabled(port)
    }
}

pub fn check_has_cra_capabilities(
    port: &mut Box<dyn SerialPort>,
    capabilites: CRACapabilityFlags,
) -> Result<bool, Box<dyn Error>> {
    let caps = commands::lock_key::capability_read(port)?;
    Ok(caps.flags.contains(capabilites))
}

pub fn update_frame_test(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    println!("Update Frame Test");

    let first_entry = [0x00, 0x50, 0x10, 0x00, 0x0A, 0x00];
    let init = BigEndian::read_u16(&first_entry);
    let mut digest = CRC16_UPDATE.digest_with_initial(init);
    digest.update(&first_entry);
    let first_entry_crc = digest.finalize();

    println!("CRC should be {:X}", first_entry_crc);

    let result = commands::update::check(port, &[0xB4, 0x8C, 0x23, 0x19, 0x5F, 0x42, 0x38])?;
    println!("Res: {result}");
    sleep(Duration::from_millis(200));

    // A972
    // ????

    // Range: Checksum: Data
    // FFFFC-FFFFD: 568D: A972
    // FFFFC-FFFFE: 0426: 000C
    // FFFFC-FFFFF: 0426: 0xA9, 0x72, 0x0C, 0xFF

    //let start = 0xC0000;
    //let end = 0xFFFFF;
    //println!("Reading data from {:05X} to {:05X}.", start, end);
    //let data = read_section(port, start, end)?;

    let start = 0xD0;
    let end = 0xDF;
    let data = read_overview(port, start, end)?;
    let mut outfile = File::create(format!("overview_{:02X}_{:02X}.bin", start, end))?;

    outfile.write_all(&data)?;

    Ok(())
}

fn read_overview(
    port: &mut Box<dyn SerialPort>,
    start: u8,
    end: u8,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut upper_addr: u8 = end;

    let mut data = Vec::new();
    data.resize(((end - start + 1) as usize * 4) as usize, 0);

    while upper_addr >= start {
        let addr: u32 = (upper_addr as u32) << 12;

        match get_dword(port, addr) {
            Ok(dword) => {
                println!("[{:05X}] {:08X}", addr, dword);
                BigEndian::write_u32(&mut data[(upper_addr - start) as usize * 4..], dword);
            }
            Err(e) => {
                println!("[{:05X}] ERR: {:?}", addr, e);
                BigEndian::write_u32(&mut data[(upper_addr - start) as usize * 4..], 0xACDCACDC);
            }
        }

        upper_addr -= 1;
    }

    Ok(data)
}

fn read_section(
    port: &mut Box<dyn SerialPort>,
    start: u32,
    end: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = Vec::new();
    data.resize((end - start + 1) as usize, 0);

    for addr in (start..=end).step_by(4) {
        match get_dword(port, addr) {
            Ok(dword) => {
                println!("[{:05X}] {:08X}", addr, dword);
                BigEndian::write_u32(&mut data[(addr - start) as usize..], dword);
            }
            Err(e) => {
                println!("[{:05X}] ERR: {:?}", addr, e);
                BigEndian::write_u32(&mut data[(addr - start) as usize..], 0xACDCACDC);
            }
        }
    }

    Ok(data)
}

fn get_dword(port: &mut Box<dyn SerialPort>, addr: u32) -> Result<u32, Box<dyn Error>> {
    let sum1 = commands::update::crc(port, addr, addr + 1)?;
    sleep(Duration::from_millis(100));
    let sum2 = commands::update::crc(port, addr, addr + 3)?;
    sleep(Duration::from_millis(100));
    let word1 = !sum1;
    let word2 = recover_second_word(word1, sum2).expect("Whoops");

    Ok(((word1 as u32) << 16) + (word2 as u32))
}
