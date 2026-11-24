//! Text and civil-time conversion at the boundary of a native Unix-microsecond instant.
use super::{
    civil_date::two_digits, CanonicalDate, CanonicalTemporalTextDenial as Denial,
    CanonicalTimestamp,
};
use std::{fmt, str::FromStr};

const MICROS_PER_SECOND: i64 = 1_000_000;
const SECONDS_PER_DAY: i64 = 86_400;
const MICROS_PER_DAY: i64 = SECONDS_PER_DAY * MICROS_PER_SECOND;

impl CanonicalTimestamp {
    /// Constructs a UTC instant from proleptic Gregorian date and time components.
    /// Rejects invalid calendar dates, leap seconds (`second >= 60`), and `micros >= 1_000_000`.
    pub fn from_ymd_hms_micros(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        micros: u32,
    ) -> Result<Self, Denial> {
        let date = CanonicalDate::from_ymd(year, month, day)?;
        if hour > 23 || minute > 59 || second > 59 || micros >= MICROS_PER_SECOND as u32 {
            return Err(Denial::InvalidTime);
        }
        let seconds = i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second);
        from_date_clock_offset(date, seconds, 0, i64::from(micros))
    }

    /// Returns the UTC proleptic Gregorian `(year, month, day, hour, minute, second, micros)`.
    pub fn to_ymd_hms_micros(self) -> (i32, u8, u8, u8, u8, u8, u32) {
        let (year, month, day) = self.utc_date().to_ymd();
        let (hour, minute, second, micros) = self.utc_clock();
        (year, month, day, hour, minute, second, micros)
    }

    /// Returns the UTC civil date of this instant.
    pub fn utc_date(self) -> CanonicalDate {
        CanonicalDate {
            // The entire i64 microsecond range occupies fewer than i32::MAX days.
            days_from_unix_epoch: self.micros_since_unix_epoch.div_euclid(MICROS_PER_DAY) as i32,
        }
    }

    /// Reads an RFC 3339 timestamp with a four-digit year and an explicit offset (`Z`, `z`, or `±HH:MM`).
    /// Leap seconds and non-zero fractional digits beyond microsecond precision refuse.
    pub fn from_rfc3339(text: &str) -> Result<Self, Denial> {
        if !matches!(text.as_bytes().get(10), Some(b'T' | b't')) {
            return Err(Denial::InvalidDate);
        }
        text.parse()
    }

    /// Formats this instant in UTC, within the four-digit RFC 3339 year range.
    pub fn to_rfc3339(self) -> Result<String, Denial> {
        if !(0..=9999).contains(&self.utc_date().to_ymd().0) {
            return Err(Denial::OutOfRange);
        }
        Ok(self.to_string())
    }

    fn utc_clock(self) -> (u8, u8, u8, u32) {
        let day_micros = self.micros_since_unix_epoch.rem_euclid(MICROS_PER_DAY);
        let seconds = day_micros / MICROS_PER_SECOND;
        let hour = (seconds / 3600) as u8;
        let minute = (seconds / 60 % 60) as u8;
        let second = (seconds % 60) as u8;
        let micros = (day_micros % MICROS_PER_SECOND) as u32;
        (hour, minute, second, micros)
    }
}

impl fmt::Display for CanonicalTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date = self.utc_date();
        let (hour, minute, second, micros) = self.utc_clock();
        write!(formatter, "{date}T{hour:02}:{minute:02}:{second:02}")?;
        if micros != 0 {
            write!(formatter, ".{micros:06}")?;
        }
        formatter.write_str("Z")
    }
}

impl FromStr for CanonicalTimestamp {
    type Err = Denial;

    /// Reads UTC (`Z`/`z`) or a numeric `±HH:MM` offset. Expanded years preserve the full native range.
    fn from_str(text: &str) -> Result<Self, Denial> {
        let (date, clock) = text.split_once(['T', 't']).ok_or(Denial::InvalidTime)?;
        let date: CanonicalDate = date.parse()?;
        let offset_start = clock
            .find(['Z', 'z', '+', '-'])
            .ok_or(Denial::InvalidTime)?;
        let (time, offset) = clock.split_at(offset_start);
        let offset = offset_seconds(offset)?;
        let (clock, micros) = match time.split_once('.') {
            Some((clock, fraction)) => (clock, fractional_micros(fraction)?),
            None => (time, 0),
        };
        let (hour, rest) = clock.split_once(':').ok_or(Denial::InvalidTime)?;
        let (minute, second) = rest.split_once(':').ok_or(Denial::InvalidTime)?;
        let seconds = clock_seconds(hour, minute)? + clock_component(second, 59)?;
        from_date_clock_offset(date, seconds, offset, micros)
    }
}

fn from_date_clock_offset(
    date: CanonicalDate,
    seconds: i64,
    offset: i64,
    micros: i64,
) -> Result<CanonicalTimestamp, Denial> {
    let absolute = i128::from(date.days_from_unix_epoch) * i128::from(MICROS_PER_DAY)
        + i128::from(seconds - offset) * i128::from(MICROS_PER_SECOND)
        + i128::from(micros);
    Ok(CanonicalTimestamp {
        micros_since_unix_epoch: i64::try_from(absolute).map_err(|_| Denial::OutOfRange)?,
    })
}

fn offset_seconds(offset: &str) -> Result<i64, Denial> {
    if matches!(offset, "Z" | "z") {
        return Ok(0);
    }
    let (sign, offset) = match offset.as_bytes().first() {
        Some(b'+') => (1, &offset[1..]),
        Some(b'-') => (-1, &offset[1..]),
        _ => return Err(Denial::InvalidTime),
    };
    let (hour, minute) = offset.split_once(':').ok_or(Denial::InvalidTime)?;
    Ok(sign * clock_seconds(hour, minute)?)
}

fn clock_seconds(hour: &str, minute: &str) -> Result<i64, Denial> {
    Ok(clock_component(hour, 23)? * 3600 + clock_component(minute, 59)? * 60)
}

fn clock_component(text: &str, max: u8) -> Result<i64, Denial> {
    two_digits(text)
        .filter(|value| *value <= max)
        .map(i64::from)
        .ok_or(Denial::InvalidTime)
}

fn fractional_micros(text: &str) -> Result<i64, Denial> {
    let mut value = 0;
    let mut place = 100_000;
    if text.is_empty() {
        return Err(Denial::InvalidTime);
    }
    for byte in text.bytes() {
        match byte {
            b'0'..=b'9' if place > 0 => {
                value += i64::from(byte - b'0') * place;
                place /= 10;
            }
            b'0' => (),
            _ => return Err(Denial::InvalidTime),
        }
    }
    Ok(value)
}
