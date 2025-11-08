use time::{format_description, OffsetDateTime, UtcOffset};

/// 获取当前 unix 时间戳（秒）
pub fn get_unix_timestamp() -> Option<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

/// 获取格式化的当前时间字符串（带时区）
/// 
/// # 参数
/// - `format_str`: 时间格式字符串（如 "[year]-[month]-[day] [hour]:[minute]:[second]"）
/// - `offset_secs`: 时区偏移（秒），如东八区为 8*3600
pub fn get_formatted_time(format_str: &str, offset_secs: i32) -> Option<String> {
    let timestamp = get_unix_timestamp()?;
    let offset = UtcOffset::from_whole_seconds(offset_secs).ok()?;
    let datetime = OffsetDateTime::from_unix_timestamp(timestamp).ok()?.to_offset(offset);
    let format = format_description::parse(format_str).ok()?;
    datetime.format(&format).ok()
}
