//! The local civil calendar day of an instant, under a fixed UTC offset or a
//! time zone's rules, with the operating system's zone and clock read only at
//! the explicit [`WorthQueryTimeZone::load_system`] / [`WorthQueryTimeZone::today`]
//! boundary.
mod posix_rule;
mod system_zone;
mod tzif;

use std::time::{SystemTime, UNIX_EPOCH};
use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};

const MICROS_PER_SECOND: i64 = 1_000_000;
/// RFC 8536 bounds a local time type's offset to `-89999..=93599` seconds.
const OFFSET_SECONDS: std::ops::RangeInclusive<i32> = -89_999..=93_599;

/// Why a local calendar date could not be determined. No variant panics or
/// falls back to UTC silently.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLocalCalendarDenial {
    /// The offset lies outside RFC 8536's `-89999..=93599` seconds.
    OffsetOutOfRange,
    /// The shifted instant lies outside the representable timestamp range.
    InstantOutOfRange,
    /// Zone rules (TZif bytes or a POSIX `TZ` rule) are malformed.
    MalformedZone,
    /// The operating system's zone could not be read.
    SystemZoneUnreadable(std::io::ErrorKind),
}

impl std::fmt::Display for WorthQueryLocalCalendarDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetOutOfRange => formatter.write_str("UTC offset is out of range"),
            Self::InstantOutOfRange => formatter.write_str("local instant is out of range"),
            Self::MalformedZone => formatter.write_str("time zone rules are malformed"),
            Self::SystemZoneUnreadable(kind) => {
                write!(formatter, "the system time zone could not be read: {kind}")
            }
        }
    }
}

impl std::error::Error for WorthQueryLocalCalendarDenial {}

/// A UTC offset in seconds east of Greenwich (UTC-7 is `-25_200`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryUtcOffset {
    seconds_east: i32,
}

impl WorthQueryUtcOffset {
    /// Coordinated Universal Time.
    pub const UTC: Self = Self { seconds_east: 0 };

    /// Admits an offset of `seconds_east` seconds east of UTC.
    pub fn from_seconds_east(seconds_east: i32) -> Result<Self, WorthQueryLocalCalendarDenial> {
        OFFSET_SECONDS
            .contains(&seconds_east)
            .then_some(Self { seconds_east })
            .ok_or(WorthQueryLocalCalendarDenial::OffsetOutOfRange)
    }

    /// Admits a whole-hour offset east of UTC (`-7` is UTC-7).
    pub fn from_hours_east(hours: i8) -> Result<Self, WorthQueryLocalCalendarDenial> {
        Self::from_seconds_east(i32::from(hours) * 3_600)
    }

    /// Seconds east of UTC.
    pub const fn seconds_east(self) -> i32 {
        self.seconds_east
    }

    /// The local civil date of `instant` at this offset.
    pub fn date_at(
        self,
        instant: CanonicalTimestamp,
    ) -> Result<CanonicalDate, WorthQueryLocalCalendarDenial> {
        let shifted = instant
            .micros_since_unix_epoch
            .checked_add(i64::from(self.seconds_east) * MICROS_PER_SECOND)
            .ok_or(WorthQueryLocalCalendarDenial::InstantOutOfRange)?;
        Ok(CanonicalTimestamp {
            micros_since_unix_epoch: shifted,
        }
        .utc_date())
    }
}

/// Time zone rules: a fixed offset, or the transitions and trailing rule of a
/// TZif zone (RFC 8536) or POSIX `TZ` rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTimeZone {
    rules: ZoneRules,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ZoneRules {
    Fixed(WorthQueryUtcOffset),
    Tzif(tzif::TzifRules),
    Posix(posix_rule::PosixRule),
}

impl WorthQueryTimeZone {
    /// The UTC zone.
    pub const UTC: Self = Self::fixed(WorthQueryUtcOffset::UTC);

    /// A zone that is always `offset` from UTC.
    pub const fn fixed(offset: WorthQueryUtcOffset) -> Self {
        Self {
            rules: ZoneRules::Fixed(offset),
        }
    }

    /// Parses TZif zone bytes (versions 1 to 4), such as a file under
    /// `/usr/share/zoneinfo`.
    pub fn from_tzif(bytes: &[u8]) -> Result<Self, WorthQueryLocalCalendarDenial> {
        tzif::TzifRules::parse(bytes).map(|rules| Self {
            rules: ZoneRules::Tzif(rules),
        })
    }

    /// Reads the operating system's zone: the `TZ` environment variable (a
    /// zone name, file path or POSIX rule) or else `/etc/localtime`. This is
    /// file I/O; load once and keep the zone.
    pub fn load_system() -> Result<Self, WorthQueryLocalCalendarDenial> {
        system_zone::load()
    }

    /// The UTC offset in effect at `instant`.
    pub fn offset_at(&self, instant: CanonicalTimestamp) -> WorthQueryUtcOffset {
        let seconds = instant
            .micros_since_unix_epoch
            .div_euclid(MICROS_PER_SECOND);
        match &self.rules {
            ZoneRules::Fixed(offset) => *offset,
            ZoneRules::Tzif(rules) => rules.offset_at(seconds),
            ZoneRules::Posix(rule) => rule.offset_at(seconds),
        }
    }

    /// The local civil date of `instant` in this zone.
    pub fn date_at(
        &self,
        instant: CanonicalTimestamp,
    ) -> Result<CanonicalDate, WorthQueryLocalCalendarDenial> {
        self.offset_at(instant).date_at(instant)
    }

    /// Today's local civil date in this zone, by the system clock.
    pub fn today(&self) -> Result<CanonicalDate, WorthQueryLocalCalendarDenial> {
        self.date_at(system_now()?)
    }

    fn posix(rule: posix_rule::PosixRule) -> Self {
        Self {
            rules: ZoneRules::Posix(rule),
        }
    }
}

/// The system clock as a canonical instant, including before the epoch.
fn system_now() -> Result<CanonicalTimestamp, WorthQueryLocalCalendarDenial> {
    let micros = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_micros()),
        Err(before) => i64::try_from(before.duration().as_micros()).map(|micros| -micros),
    }
    .map_err(|_| WorthQueryLocalCalendarDenial::InstantOutOfRange)?;
    Ok(CanonicalTimestamp {
        micros_since_unix_epoch: micros,
    })
}
