//! An order-preserving `u64` image of a grant validity value, so durable
//! records written with a `u64` expiry hold every timeline without a new wire
//! shape. Values compare only within one timeline; records carry its name.
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline as Timeline;

const SIGN_FLIP: u64 = 1 << 63;

/// The ordinal of `value` on `timeline`, or `None` when the value's family is
/// not the timeline's. Epoch-count values are their own ordinal, so every
/// existing record is unchanged; signed timelines flip the sign bit of the
/// value widened to `i64`, which maps signed order onto unsigned order.
pub fn validity_ordinal(timeline: Timeline, value: &AspectValue) -> Option<u64> {
    match (timeline, value) {
        (
            Timeline::UnixEpochSeconds | Timeline::UnixEpochMilliseconds,
            AspectValue::UInt64(units),
        ) => Some(*units),
        (Timeline::UtcTimestamp, AspectValue::Timestamp(instant)) => {
            Some(flip_sign(instant.micros_since_unix_epoch))
        }
        (Timeline::LocalDate, AspectValue::Date(date)) => {
            Some(flip_sign(i64::from(date.days_from_unix_epoch)))
        }
        _ => None,
    }
}

fn flip_sign(value: i64) -> u64 {
    u64::from_ne_bytes(value.to_ne_bytes()) ^ SIGN_FLIP
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};

    fn instant(micros: i64) -> AspectValue {
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: micros,
        })
    }

    fn date(days: i32) -> AspectValue {
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: days,
        })
    }

    #[test]
    fn ordinals_preserve_order_across_the_epoch_and_the_full_signed_range() {
        let instants = [i64::MIN, -86_400_000_001, -1, 0, 1, i64::MAX];
        let ordinals =
            instants.map(|micros| validity_ordinal(Timeline::UtcTimestamp, &instant(micros)));
        assert!(ordinals.windows(2).all(|pair| pair[0] < pair[1]));
        let days = [i32::MIN, -719_468, -1, 0, 1, i32::MAX];
        let ordinals = days.map(|day| validity_ordinal(Timeline::LocalDate, &date(day)));
        assert!(ordinals.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn epoch_count_ordinals_are_the_stored_value_and_families_must_match() {
        assert_eq!(
            validity_ordinal(Timeline::UnixEpochSeconds, &AspectValue::UInt64(42)),
            Some(42)
        );
        assert_eq!(validity_ordinal(Timeline::LocalDate, &instant(0)), None);
        assert_eq!(
            validity_ordinal(Timeline::UtcTimestamp, &AspectValue::UInt64(0)),
            None
        );
    }
}
