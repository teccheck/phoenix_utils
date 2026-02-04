use std::error::Error;

use chrono::NaiveDateTime;
use serialport::SerialPort;

use crate::{
    cli::types::{BacklightMode, LedMode, PagerKey},
    phoenix::{
        self,
        types::{
            FeatureFlag, FeatureFlagNotFoundError, StorageBlockId, StorageBlockLength,
            StorageBlockOffset,
        },
    },
};

pub fn print_device_info(port: &mut Box<dyn SerialPort>) {
    match phoenix::tasks::read_device_info(port) {
        Ok(info) => println!("{}", info),
        Err(e) => println!("Error reading device info: {}", e),
    }
}

pub fn print_storage_directory(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    println!("Reading Storage directory. This might take a few seconds...");
    let dir = phoenix::tasks::read_storage_directory(port)?;

    println!("| ID   | Version | Size   | Flags |");
    for block in dir {
        println!(
            "| {:>4x} | {:>7} | {:>6} | {:>5} |",
            block.id,
            block.version,
            block.length,
            block.permissions.flag_string()
        );
    }

    Ok(())
}

pub fn print_storage_block(
    port: &mut Box<dyn SerialPort>,
    id: StorageBlockId,
    offset: StorageBlockOffset,
    length: StorageBlockLength,
) -> Result<(), Box<dyn Error>> {
    let data = phoenix::tasks::read_storage_block(port, id, offset, length)?;
    println!("Storage Block ({:X}): {:X?}", id, data);
    Ok(())
}

pub fn cra_read_capabilities(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let capabilities = phoenix::commands::lock_key::cra_capability_read(port)?;
    println!("Capabilities:\n{}", capabilities);
    Ok(())
}

pub fn feature_flags_read_enabled(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let flags = phoenix::tasks::feature_flags_read_enabled(port)?;
    println!("Enabled flags: [{}]", flags);
    Ok(())
}

pub fn feature_flags_read_supported(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let flags = phoenix::commands::feature_flags::read_supported(port)?;
    println!("Supported flags: [{}]", flags);
    Ok(())
}

pub fn feature_flags_write(
    port: &mut Box<dyn SerialPort>,
    flags: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let new_flags = parse_flags_vec(flags)?;
    println!("Write Feature Flags: [{}]", new_flags);

    phoenix::commands::sys::write_feature_flags(port, new_flags)
}

fn parse_flags_vec(flags: Vec<String>) -> Result<FeatureFlag, Box<dyn Error>> {
    let new_flags: Result<Vec<FeatureFlag>, FeatureFlagNotFoundError> =
        flags.iter().map(find_feature_flag_by_string).collect();

    let new_flags = new_flags?
        .into_iter()
        .reduce(FeatureFlag::or)
        .unwrap_or_else(FeatureFlag::none);

    Ok(new_flags)
}

fn find_feature_flag_by_string(flag: &String) -> Result<FeatureFlag, FeatureFlagNotFoundError> {
    FeatureFlag::flags()
        .find(|(n, _)| n.eq(flag))
        .map(|(_, f)| *f)
        .ok_or(FeatureFlagNotFoundError {
            flag_name: flag.to_string(),
        })
}

pub fn led_mode(port: &mut Box<dyn SerialPort>, mode: LedMode) -> Result<(), Box<dyn Error>> {
    if matches!(mode, LedMode::Normal) {
        phoenix::commands::tools::led_normal_mode(port)?;
    } else {
        phoenix::commands::tools::led_test_mode(port, mode as u8)?;
    }

    Ok(())
}

pub fn backlight_mode(
    port: &mut Box<dyn SerialPort>,
    mode: BacklightMode,
) -> Result<(), Box<dyn Error>> {
    if matches!(mode, BacklightMode::Normal) {
        phoenix::commands::tools::backlight_normal_mode(port)?;
    } else {
        phoenix::commands::tools::backlight_test_mode(port, mode as u8)?;
    }

    Ok(())
}

pub fn key_press(port: &mut Box<dyn SerialPort>, key: PagerKey) -> Result<(), Box<dyn Error>> {
    phoenix::commands::key_press(port, key as u8)
}

pub fn key_release(port: &mut Box<dyn SerialPort>, key: PagerKey) -> Result<(), Box<dyn Error>> {
    phoenix::commands::key_release(port, key as u8)
}

pub fn time_set(
    port: &mut Box<dyn SerialPort>,
    time: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let datetime = if let Some(time) = time {
        NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S")?
    } else {
        chrono::offset::Local::now().naive_utc()
    };

    phoenix::commands::time::set_utc(port, &datetime)?;
    Ok(())
}

pub fn time_get(port: &mut Box<dyn SerialPort>, utc: bool) -> Result<(), Box<dyn Error>> {
    let datetime = if utc {
        phoenix::commands::time::get_utc(port)
    } else {
        phoenix::commands::time::get_local(port)
    }?;

    println!("Time on device is {datetime}");
    Ok(())
}
