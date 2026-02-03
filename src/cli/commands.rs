use std::error::Error;

use serialport::SerialPort;

use crate::{
    cli::types::{BacklightMode, LedMode, PagerKey},
    phoenix::{
        commands::{
            command_backlight_normal_mode, command_backlight_test_mode, command_key_press, command_key_release, command_led_normal_mode, command_led_test_mode, command_write_feature_flags
        },
        types::{FeatureFlag, FeatureFlagNotFoundError},
    },
};

pub fn write_feature_flags(
    port: &mut Box<dyn SerialPort>,
    flags: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let new_flags = parse_flags_vec(flags)?;
    println!("Write Feature Flags: [{}]", new_flags);

    command_write_feature_flags(port, new_flags)
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
        command_led_normal_mode(port)?;
    } else {
        command_led_test_mode(port, mode as u8)?;
    }

    Ok(())
}

pub fn backlight_mode(
    port: &mut Box<dyn SerialPort>,
    mode: BacklightMode,
) -> Result<(), Box<dyn Error>> {
    if matches!(mode, BacklightMode::Normal) {
        command_backlight_normal_mode(port)?;
    } else {
        command_backlight_test_mode(port, mode as u8)?;
    }

    Ok(())
}

pub fn key_press(port: &mut Box<dyn SerialPort>, key: PagerKey) -> Result<(), Box<dyn Error>> {
    command_key_press(port, key as u8)
}

pub fn key_release(port: &mut Box<dyn SerialPort>, key: PagerKey) -> Result<(), Box<dyn Error>> {
    command_key_release(port, key as u8)
}
