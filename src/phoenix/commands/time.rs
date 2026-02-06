use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use serialport::SerialPort;

use crate::phoenix::{
    commands::{
        send_command, check_response_result_simple_inv, check_response_type,
    },
    types::CommandType,
};

pub fn set_utc(
    port: &mut Box<dyn SerialPort>,
    datetime: &NaiveDateTime,
) -> Result<(), Box<dyn Error>> {
    let mut args = [0_u8; 7];
    write_time(&mut args, datetime);
    let rsp = send_command(port, CommandType::TimeSet, &args)?;
    let rsp = check_response_type(&rsp, CommandType::TimeSet)?;
    check_response_result_simple_inv(rsp, "time_set_utc")?;
    Ok(())
}

pub fn get_utc(port: &mut Box<dyn SerialPort>) -> Result<NaiveDateTime, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::TimeGetUtc, &[])?;
    let rsp = check_response_type(&rsp, CommandType::TimeGetUtc)?;
    Ok(read_time(rsp))
}

pub fn get_local(port: &mut Box<dyn SerialPort>) -> Result<NaiveDateTime, Box<dyn Error>> {
    let rsp = send_command(port, CommandType::TimeGetLocal, &[])?;
    let rsp = check_response_type(&rsp, CommandType::TimeGetLocal)?;
    Ok(read_time(rsp))
}

fn read_time(data: &[u8]) -> NaiveDateTime {
    let year = BigEndian::read_u16(&data[5..]);
    let month = data[4];
    let day = data[3];
    let hour = data[0];
    let min = data[1];
    let sec = data[2];

    let time = NaiveTime::from_hms_opt(hour.into(), min.into(), sec.into()).unwrap();
    NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())
        .unwrap()
        .and_time(time)
}

fn write_time(data: &mut [u8], datetime: &NaiveDateTime) {
    data[0] = datetime.hour() as u8;
    data[1] = datetime.minute() as u8;
    data[2] = datetime.second() as u8;
    data[3] = datetime.day() as u8;
    data[4] = datetime.month() as u8;
    BigEndian::write_u16(&mut data[5..], datetime.year() as u16);
}
