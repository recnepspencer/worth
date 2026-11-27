//! Exact conversion between a trusted `SystemTime` reading and a canonical
//! microsecond UTC timestamp. Instants before the Unix epoch are negative;
//! sub-microsecond precision is truncated toward the epoch; an instant either
//! side cannot represent is refused, never clamped.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use worth_foundational::facade::CanonicalTimestamp;

use super::time_basis::WorthQueryRuntimeTimeDenial;

pub(super) fn timestamp_of(
    instant: SystemTime,
) -> Result<CanonicalTimestamp, WorthQueryRuntimeTimeDenial> {
    let micros = match instant.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_micros()),
        Err(before) => i64::try_from(before.duration().as_micros()).map(|micros| -micros),
    }
    .map_err(|_| WorthQueryRuntimeTimeDenial::TimelineValueOverflow)?;
    Ok(CanonicalTimestamp {
        micros_since_unix_epoch: micros,
    })
}

pub(super) fn system_time_of(timestamp: CanonicalTimestamp) -> Option<SystemTime> {
    let micros = timestamp.micros_since_unix_epoch;
    let magnitude = Duration::from_micros(micros.unsigned_abs());
    if micros >= 0 {
        UNIX_EPOCH.checked_add(magnitude)
    } else {
        UNIX_EPOCH.checked_sub(magnitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(micros: i64) -> CanonicalTimestamp {
        CanonicalTimestamp {
            micros_since_unix_epoch: micros,
        }
    }

    #[test]
    fn instants_round_trip_on_both_sides_of_the_epoch() {
        for micros in [0, 1, -1, 1_795_829_400_000_000, -86_400_000_001] {
            let instant = system_time_of(at(micros)).expect("the platform represents it");
            assert_eq!(timestamp_of(instant), Ok(at(micros)));
        }
    }

    #[test]
    fn sub_microsecond_precision_truncates_toward_the_epoch() {
        let after = UNIX_EPOCH + Duration::from_nanos(1_999);
        assert_eq!(timestamp_of(after), Ok(at(1)));
        let before = UNIX_EPOCH
            .checked_sub(Duration::from_nanos(1_999))
            .expect("the platform represents a pre-epoch instant");
        assert_eq!(timestamp_of(before), Ok(at(-1)));
    }
}
