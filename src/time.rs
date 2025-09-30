use color_eyre::eyre::anyhow;
use color_eyre::Result;
use time::OffsetDateTime;
use time::format_description::{BorrowedFormatItem, parse as parse_fmt, FormatItem};
use time::macros::format_description;

const TEAMCITY_DATETIME_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]T[hour][minute][second][optional [.[subsecond]]][offset_hour sign:mandatory][offset_minute]");
const HUMAN_READABLE_DATE_FORMAT: &[FormatItem<'static>] = format_description!("[day] [month repr:short] [hour repr:24]:[minute]");
const DURATION_TIME_FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

pub fn format_datetime_to_human_readable_string(date: &str) -> Result<String> {
    let datetime = OffsetDateTime::parse(date, &TEAMCITY_DATETIME_FORMAT)?;

    datetime.format(&HUMAN_READABLE_DATE_FORMAT)
        .map_err(|e| anyhow!(e))
}

// Parse TeamCity datetime like "YYYYMMDDTHHMMSS+0000" (optionally with fractional seconds) into unix seconds
pub fn parse_tc_datetime_to_epoch(s: &str) -> Result<i64> {
    OffsetDateTime::parse(s, &TEAMCITY_DATETIME_FORMAT)
        .map(|dt| dt.unix_timestamp())
        .map_err(|e| anyhow!(e))
}

pub fn format_duration(secs: i64) -> Result<String> {
    let datetime = OffsetDateTime::from_unix_timestamp(secs)?;

    datetime.format(&DURATION_TIME_FORMAT)
        .map_err(|e| anyhow!(e))
}