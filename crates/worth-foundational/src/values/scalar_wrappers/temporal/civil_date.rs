//! Proleptic Gregorian conversion over the complete native day range.
use super::{
    timestamp_text::MICROS_PER_DAY, CanonicalDate, CanonicalTemporalTextDenial as Denial,
    CanonicalTimestamp,
};
use std::{fmt, str::FromStr};

const DAYS_PER_ERA: i64 = 146_097;
const UNIX_EPOCH_FROM_MARCH_0000: i64 = 719_468;

impl CanonicalDate {
    /// Parses a canonical ISO-8601 civil date (`YYYY-MM-DD`, or expanded signed year).
    pub fn parse_iso(text: &str) -> Result<Self, Denial> {
        text.parse()
    }

    /// Shifts this date by `days` on the proleptic Gregorian calendar, failing
    /// closed with `OutOfRange` on overflow.
    pub fn add_days(self, days: i64) -> Result<Self, Denial> {
        let shifted = i64::from(self.days_from_unix_epoch)
            .checked_add(days)
            .ok_or(Denial::OutOfRange)?;
        Ok(Self {
            days_from_unix_epoch: i32::try_from(shifted).map_err(|_| Denial::OutOfRange)?,
        })
    }

    /// Returns midnight (`00:00:00Z`) at the start of this UTC civil date,
    /// failing closed with `OutOfRange` when outside the `i64` microsecond range.
    pub fn start_of_day_utc(&self) -> Result<CanonicalTimestamp, Denial> {
        let micros = i64::from(self.days_from_unix_epoch)
            .checked_mul(MICROS_PER_DAY)
            .ok_or(Denial::OutOfRange)?;
        Ok(CanonicalTimestamp {
            micros_since_unix_epoch: micros,
        })
    }

    /// Constructs a date in the proleptic Gregorian calendar, with a year zero.
    pub fn from_ymd(year: i32, month: u8, day: u8) -> Result<Self, Denial> {
        let maximum_day = match month {
            2 if is_leap_year(year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => return Err(Denial::InvalidDate),
        };
        if day == 0 || day > maximum_day {
            return Err(Denial::InvalidDate);
        }

        // The March-based Gregorian eras follow Howard Hinnant's date algorithms.
        // https://howardhinnant.github.io/date_algorithms.html#days_from_civil
        let year = i64::from(year) - i64::from(month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year.rem_euclid(400);
        let month_from_march = i64::from((month + 9) % 12);
        let day_of_year = (153 * month_from_march + 2) / 5 + i64::from(day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days = era * DAYS_PER_ERA + day_of_era - UNIX_EPOCH_FROM_MARCH_0000;
        Ok(Self {
            days_from_unix_epoch: i32::try_from(days).map_err(|_| Denial::OutOfRange)?,
        })
    }

    /// Returns the Gregorian year, month and day, including for pre-epoch dates.
    pub fn to_ymd(self) -> (i32, u8, u8) {
        let days = i64::from(self.days_from_unix_epoch) + UNIX_EPOCH_FROM_MARCH_0000;
        let era = days.div_euclid(DAYS_PER_ERA);
        let day_of_era = days.rem_euclid(DAYS_PER_ERA);
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_from_march = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
        let month = (month_from_march + 2) % 12 + 1;
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        (year as i32, month as u8, day as u8)
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

impl fmt::Display for CanonicalDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = self.to_ymd();
        if (0..=9999).contains(&year) {
            write!(formatter, "{year:04}-{month:02}-{day:02}")
        } else {
            write!(formatter, "{year:+07}-{month:02}-{day:02}")
        }
    }
}

impl FromStr for CanonicalDate {
    type Err = Denial;

    fn from_str(text: &str) -> Result<Self, Denial> {
        let (year_month, day) = text.rsplit_once('-').ok_or(Denial::InvalidDate)?;
        let (year, month) = year_month.rsplit_once('-').ok_or(Denial::InvalidDate)?;
        let signed = year.starts_with(['+', '-']);
        let digits = if signed { &year[1..] } else { year };
        if (signed && digits.len() < 6)
            || (!signed && digits.len() != 4)
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(Denial::InvalidDate);
        }
        let month = two_digits(month).ok_or(Denial::InvalidDate)?;
        let day = two_digits(day).ok_or(Denial::InvalidDate)?;
        let year = year.parse::<i32>().map_err(|_| Denial::OutOfRange)?;
        if signed && (0..=9999).contains(&year) {
            return Err(Denial::InvalidDate);
        }
        Self::from_ymd(year, month, day)
    }
}

pub(super) fn two_digits(text: &str) -> Option<u8> {
    match text.as_bytes() {
        [first @ b'0'..=b'9', second @ b'0'..=b'9'] => Some((first - b'0') * 10 + second - b'0'),
        _ => None,
    }
}
