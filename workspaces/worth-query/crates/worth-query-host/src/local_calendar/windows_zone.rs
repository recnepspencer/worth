//! Windows zone boundary: the system zone's rules from the registry key
//! `HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation`.
//!
//! Windows has no TZif files. Its key holds the current zone as a bias and
//! two yearly `SYSTEMTIME` transitions, the shape of a POSIX `Mm.w.d` rule.
//! The values are read by `reg.exe`, the operating system's own registry
//! tool, so no unsafe platform call is needed; parsing its export is
//! portable and tested on every platform.
use super::posix_rule::{MonthWeekDayTransition, PosixRule};
use super::{WorthQueryLocalCalendarDenial as Denial, WorthQueryTimeZone, WorthQueryUtcOffset};

/// The key `reg query` reads.
#[cfg(windows)]
const ZONE_KEY: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation";

/// Reads the system zone through `%SystemRoot%\System32\reg.exe`.
#[cfg(windows)]
pub(super) fn load() -> Result<WorthQueryTimeZone, Denial> {
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let tool = std::path::Path::new(&system_root).join(r"System32\reg.exe");
    let output = std::process::Command::new(tool)
        .args(["query", ZONE_KEY])
        .output()
        .map_err(|cause| Denial::SystemZoneUnreadable(cause.kind()))?;
    if !output.status.success() {
        return Err(Denial::SystemZoneUnreadable(std::io::ErrorKind::NotFound));
    }
    from_registry_query(&String::from_utf8_lossy(&output.stdout))
}

/// The zone a `reg query` export of the key describes. `Bias`, the two
/// biases and both `*Start` values are required; a zone with no daylight
/// period, or with dynamic daylight time disabled, is fixed at standard time.
pub(super) fn from_registry_query(export: &str) -> Result<WorthQueryTimeZone, Denial> {
    let dword = |name| registry_value(export, name, "REG_DWORD").and_then(parse_dword);
    let start = |name| registry_value(export, name, "REG_BINARY").and_then(parse_system_time);
    let bias = dword("Bias").ok_or(Denial::MalformedZone)?;
    let standard_bias = dword("StandardBias").ok_or(Denial::MalformedZone)?;
    let daylight_bias = dword("DaylightBias").ok_or(Denial::MalformedZone)?;
    let standard_start = start("StandardStart").ok_or(Denial::MalformedZone)?;
    let daylight_start = start("DaylightStart").ok_or(Denial::MalformedZone)?;
    let standard = offset_east(bias, standard_bias)?;
    let daylight_disabled = dword("DynamicDaylightTimeDisabled").is_some_and(|flag| flag != 0);
    if daylight_disabled || standard_start.month == 0 || daylight_start.month == 0 {
        return Ok(WorthQueryTimeZone::fixed(standard));
    }
    PosixRule::month_week_day(
        standard,
        offset_east(bias, daylight_bias)?,
        daylight_start.transition()?,
        standard_start.transition()?,
    )
    .map(WorthQueryTimeZone::posix)
}

/// The registry's bias is minutes west of UTC (`UTC = local + bias`).
fn offset_east(bias: i32, extra: i32) -> Result<WorthQueryUtcOffset, Denial> {
    let minutes_west = bias.checked_add(extra).ok_or(Denial::OffsetOutOfRange)?;
    let seconds_east = minutes_west
        .checked_mul(-60)
        .ok_or(Denial::OffsetOutOfRange)?;
    WorthQueryUtcOffset::from_seconds_east(seconds_east)
}

/// The data of value `name` with registry type `kind`: a line
/// `    name    kind    data`.
fn registry_value<'export>(export: &'export str, name: &str, kind: &str) -> Option<&'export str> {
    export.lines().find_map(|line| {
        let mut columns = line.split_whitespace();
        (columns.next() == Some(name) && columns.next() == Some(kind))
            .then(|| columns.next())
            .flatten()
    })
}

/// `0x...`: a 32-bit value; biases are signed, so `0xffffffc4` is -60.
fn parse_dword(data: &str) -> Option<i32> {
    let digits = data.strip_prefix("0x")?;
    u32::from_str_radix(digits, 16)
        .ok()
        .map(|value| i32::from_ne_bytes(value.to_ne_bytes()))
}

/// The `SYSTEMTIME` fields a yearly transition uses.
struct SystemTime {
    year: u16,
    month: u16,
    weekday: u16,
    week: u16,
    seconds: i64,
}

impl SystemTime {
    /// A yearly rule has year 0; a dated, one-time transition is refused.
    /// Milliseconds round up, so `23:59:59.999` is the next midnight.
    fn transition(&self) -> Result<MonthWeekDayTransition, Denial> {
        let narrow = |value: u16| u8::try_from(value).map_err(|_| Denial::MalformedZone);
        if self.year != 0 {
            return Err(Denial::MalformedZone);
        }
        Ok(MonthWeekDayTransition {
            month: narrow(self.month)?,
            week: narrow(self.week)?,
            weekday: narrow(self.weekday)?,
            local_second: self.seconds,
        })
    }
}

/// 16 little-endian `u16` fields as 32 hex digits: year, month, weekday,
/// day (the week of the month, 5 for the last), hour, minute, second and
/// milliseconds.
fn parse_system_time(data: &str) -> Option<SystemTime> {
    if data.len() != 32 || !data.is_ascii() {
        return None;
    }
    let mut fields = [0_u16; 8];
    for (index, field) in fields.iter_mut().enumerate() {
        let low = u8::from_str_radix(data.get(index * 4..index * 4 + 2)?, 16).ok()?;
        let high = u8::from_str_radix(data.get(index * 4 + 2..index * 4 + 4)?, 16).ok()?;
        *field = u16::from_le_bytes([low, high]);
    }
    let [year, month, weekday, week, hour, minute, second, millis] = fields;
    let seconds = i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second)
        + i64::from(millis > 0);
    Some(SystemTime {
        year,
        month,
        weekday,
        week,
        seconds,
    })
}
