//! POSIX `TZ` rules (`std offset [dst [offset] [,start[/time],end[/time]]]`),
//! as in a TZif footer or the `TZ` environment variable.
use super::{WorthQueryLocalCalendarDenial as Denial, WorthQueryUtcOffset};
use worth_foundational::facade::CanonicalDate;

const SECONDS_PER_DAY: i64 = 86_400;
/// glibc's default when a rule names daylight time without transitions.
const DEFAULT_TRANSITIONS: &str = ",M3.2.0,M11.1.0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PosixRule {
    standard: WorthQueryUtcOffset,
    daylight: Option<Daylight>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Daylight {
    offset: WorthQueryUtcOffset,
    start: Transition,
    end: Transition,
}

/// A yearly transition at `local_second` of a day (RFC 8536 allows ±167h).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Transition {
    day: DayRule,
    local_second: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DayRule {
    /// `Jn`: day 1..=365, never counting February 29.
    JulianNoLeap(u16),
    /// `n`: zero-based day 0..=365, counting February 29.
    ZeroBased(u16),
    /// `Mm.w.d`: weekday `d` (0 = Sunday) of week `w` (5 = last) of month `m`.
    MonthWeekDay { month: u8, week: u8, weekday: u8 },
}

impl PosixRule {
    pub(super) fn parse(text: &str) -> Result<Self, Denial> {
        let mut cursor = RuleCursor(text.as_bytes());
        cursor.name()?;
        let standard = west_to_east(cursor.offset(24)?)?;
        if cursor.0.is_empty() {
            return Ok(Self {
                standard,
                daylight: None,
            });
        }
        cursor.name()?;
        let offset = match cursor.0.first() {
            Some(b'0'..=b'9' | b'+' | b'-') => west_to_east(cursor.offset(24)?)?,
            _ => WorthQueryUtcOffset::from_seconds_east(standard.seconds_east() + 3_600)?,
        };
        if cursor.0.is_empty() {
            cursor = RuleCursor(DEFAULT_TRANSITIONS.as_bytes());
        }
        let start = cursor.transition()?;
        let end = cursor.transition()?;
        if !cursor.0.is_empty() {
            return Err(Denial::MalformedZone);
        }
        Ok(Self {
            standard,
            daylight: Some(Daylight { offset, start, end }),
        })
    }

    pub(super) fn offset_at(&self, second: i64) -> WorthQueryUtcOffset {
        let Some(daylight) = &self.daylight else {
            return self.standard;
        };
        let local = second.saturating_add(i64::from(self.standard.seconds_east()));
        let year = year_of(local);
        // A start is expressed in standard time and an end in daylight time.
        let (Some(start), Some(end)) = (
            daylight.start.utc_second(year, self.standard),
            daylight.end.utc_second(year, daylight.offset),
        ) else {
            return self.standard;
        };
        let in_daylight = if start <= end {
            start <= second && second < end
        } else {
            !(end <= second && second < start)
        };
        if in_daylight {
            daylight.offset
        } else {
            self.standard
        }
    }
}

impl Transition {
    fn utc_second(self, year: i32, offset: WorthQueryUtcOffset) -> Option<i64> {
        let day = i64::from(self.day.day_in(year)?.days_from_unix_epoch);
        (day * SECONDS_PER_DAY + self.local_second).checked_sub(i64::from(offset.seconds_east()))
    }
}

impl DayRule {
    fn day_in(self, year: i32) -> Option<CanonicalDate> {
        let january_first = CanonicalDate::from_ymd(year, 1, 1).ok()?;
        match self {
            Self::JulianNoLeap(day) => {
                let leap_shift = u16::from(day >= 60 && is_leap(year));
                january_first.add_days(i64::from(day - 1 + leap_shift)).ok()
            }
            Self::ZeroBased(day) => january_first.add_days(i64::from(day)).ok(),
            Self::MonthWeekDay {
                month,
                week,
                weekday,
            } => {
                let first = CanonicalDate::from_ymd(year, month, 1).ok()?;
                // 1970-01-01 was a Thursday (weekday 4).
                let first_weekday = (i64::from(first.days_from_unix_epoch) + 4).rem_euclid(7);
                let mut day =
                    (i64::from(weekday) - first_weekday).rem_euclid(7) + 7 * i64::from(week - 1);
                while day >= i64::from(days_in_month(year, month)) {
                    day -= 7;
                }
                first.add_days(day).ok()
            }
        }
    }
}

struct RuleCursor<'text>(&'text [u8]);

impl RuleCursor<'_> {
    /// An alphabetic name of at least three letters, or a `<quoted>` name.
    fn name(&mut self) -> Result<(), Denial> {
        let length = if let Some(rest) = self.0.strip_prefix(b"<") {
            let close = rest.iter().position(|&byte| byte == b'>');
            close
                .filter(|&close| close > 0)
                .ok_or(Denial::MalformedZone)?
                + 2
        } else {
            let letters = self.0.iter().take_while(|byte| byte.is_ascii_alphabetic());
            Some(letters.count())
                .filter(|&count| count >= 3)
                .ok_or(Denial::MalformedZone)?
        };
        self.0 = &self.0[length..];
        Ok(())
    }

    /// `[+-]hh[:mm[:ss]]` with hours at most `max_hours`, in seconds.
    fn offset(&mut self, max_hours: i64) -> Result<i64, Denial> {
        let sign = match self.0.first() {
            Some(b'-') => -1,
            _ => 1,
        };
        if matches!(self.0.first(), Some(b'+' | b'-')) {
            self.0 = &self.0[1..];
        }
        let hours = self.number(3)?;
        let mut seconds = hours * 3_600;
        for unit in [60, 1] {
            if self.0.first() != Some(&b':') {
                break;
            }
            self.0 = &self.0[1..];
            let part = self.number(2)?;
            if part > 59 {
                return Err(Denial::MalformedZone);
            }
            seconds += part * unit;
        }
        (hours <= max_hours)
            .then_some(sign * seconds)
            .ok_or(Denial::MalformedZone)
    }

    /// `,rule[/time]`.
    fn transition(&mut self) -> Result<Transition, Denial> {
        self.0 = self.0.strip_prefix(b",").ok_or(Denial::MalformedZone)?;
        let day = match self.0.first() {
            Some(b'J') => {
                self.0 = &self.0[1..];
                DayRule::JulianNoLeap(self.bounded(1..=365)?)
            }
            Some(b'M') => {
                self.0 = &self.0[1..];
                let month = self.bounded(1..=12)?;
                self.0 = self.0.strip_prefix(b".").ok_or(Denial::MalformedZone)?;
                let week = self.bounded(1..=5)?;
                self.0 = self.0.strip_prefix(b".").ok_or(Denial::MalformedZone)?;
                let weekday = self.bounded(0..=6)?;
                DayRule::MonthWeekDay {
                    month,
                    week,
                    weekday,
                }
            }
            _ => DayRule::ZeroBased(self.bounded(0..=365)?),
        };
        let local_second = match self.0.strip_prefix(b"/") {
            Some(rest) => {
                self.0 = rest;
                self.offset(167)?
            }
            None => 7_200,
        };
        Ok(Transition { day, local_second })
    }

    fn bounded<T: TryFrom<i64>>(
        &mut self,
        range: std::ops::RangeInclusive<i64>,
    ) -> Result<T, Denial> {
        let value = self.number(3)?;
        range
            .contains(&value)
            .then(|| T::try_from(value).ok())
            .flatten()
            .ok_or(Denial::MalformedZone)
    }

    /// One to `max_digits` decimal digits.
    fn number(&mut self, max_digits: usize) -> Result<i64, Denial> {
        let digits = self
            .0
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digits == 0 || digits > max_digits {
            return Err(Denial::MalformedZone);
        }
        let value = self.0[..digits]
            .iter()
            .fold(0_i64, |value, digit| value * 10 + i64::from(digit - b'0'));
        self.0 = &self.0[digits..];
        Ok(value)
    }
}

/// POSIX offsets count hours west of UTC (`MST7` is UTC-7).
fn west_to_east(seconds_west: i64) -> Result<WorthQueryUtcOffset, Denial> {
    let seconds_east = i32::try_from(-seconds_west).map_err(|_| Denial::MalformedZone)?;
    WorthQueryUtcOffset::from_seconds_east(seconds_east)
}

fn year_of(local_second: i64) -> i32 {
    let days = local_second.div_euclid(SECONDS_PER_DAY);
    let days = i32::try_from(days).unwrap_or(if days < 0 { i32::MIN } else { i32::MAX });
    CanonicalDate {
        days_from_unix_epoch: days,
    }
    .to_ymd()
    .0
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
