use std::error::Error;

use serialport::SerialPort;

use crate::phoenix::{
    commands::command_write_feature_flags,
    types::{FeatureFlag, FeatureFlagNotFoundError},
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
