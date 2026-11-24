use serde::{Deserialize, Serialize};

mod civil_date;
#[cfg(test)]
mod tests;
mod timestamp_text;

/// A civil date or timestamp could not be represented by the native scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalTemporalTextDenial {
    InvalidDate,
    InvalidTime,
    OutOfRange,
}

impl std::fmt::Display for CanonicalTemporalTextDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDate => "invalid Gregorian date",
            Self::InvalidTime => "invalid time or UTC offset",
            Self::OutOfRange => "date or time is outside the representable range",
        })
    }
}

impl std::error::Error for CanonicalTemporalTextDenial {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalDate {
    pub days_from_unix_epoch: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalTime {
    pub nanos_since_midnight: u64,
}

impl CanonicalTime {
    pub const NANOS_PER_DAY: u64 = 86_400_000_000_000;

    pub fn new(nanos_since_midnight: u64) -> Option<Self> {
        if nanos_since_midnight < Self::NANOS_PER_DAY {
            Some(Self {
                nanos_since_midnight,
            })
        } else {
            None
        }
    }

    pub(crate) fn is_canonical(self) -> bool {
        self.nanos_since_midnight < Self::NANOS_PER_DAY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalTimestamp {
    pub micros_since_unix_epoch: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalTimestampTz {
    pub utc_micros_since_unix_epoch: i64,
    pub offset_minutes: i32,
}

impl CanonicalTimestampTz {
    pub(crate) fn is_canonical(self) -> bool {
        (-1_439..=1_439).contains(&self.offset_minutes)
    }
}
