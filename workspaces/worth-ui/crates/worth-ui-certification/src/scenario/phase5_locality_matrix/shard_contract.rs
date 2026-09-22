use super::case::Phase5LocalityAxis;
use super::RETAINED_SIZES;

pub(super) const ROW_COUNT: usize = RETAINED_SIZES.len() * Phase5LocalityAxis::ALL.len();
/// Every shard runs one row from a smoke world beside one from a large world,
/// so the matrix governs half as many shards as it has rows. Deriving that
/// count keeps the pairing exact when the axis list changes, which a governed
/// constant did not: removing one axis left sixteen shards dividing
/// twenty-eight rows, and four of them owned a single row.
pub(super) const SHARD_COUNT: usize = ROW_COUNT / 2;

pub(super) fn report_name(shard: usize) -> String {
    format!("worth-ui-phase5-locality-{shard}.jsonl")
}

pub(super) fn expected_rows(shard: usize) -> usize {
    (0..ROW_COUNT)
        .filter(|ordinal| ordinal % SHARD_COUNT == shard)
        .count()
}

pub(super) fn is_large(shard: usize) -> bool {
    shard >= SHARD_COUNT / 2
}

pub(super) fn validate_shard(shard: usize, count: usize) -> Result<(), String> {
    if count != SHARD_COUNT {
        return Err(format!(
            "matrix shard count {count} does not match governed count {SHARD_COUNT}"
        ));
    }
    if shard >= count {
        return Err(format!("matrix shard {shard}/{count} is out of range"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{expected_rows, is_large, ROW_COUNT, SHARD_COUNT};

    const LARGE_FLOOR: usize = SHARD_COUNT / 2;

    #[test]
    fn every_shard_owns_exactly_two_rows() {
        assert_eq!((0..SHARD_COUNT).map(expected_rows).sum::<usize>(), ROW_COUNT);
        assert!((0..SHARD_COUNT).all(|shard| expected_rows(shard) == 2));
    }

    #[test]
    fn only_the_four_thousand_ninety_six_shards_are_large() {
        assert!((0..LARGE_FLOOR).all(|shard| !is_large(shard)));
        assert!((LARGE_FLOOR..SHARD_COUNT).all(is_large));
    }

    #[test]
    fn each_shard_pairs_one_large_or_one_middle_world_with_a_smoke_world() {
        let large_rows = (LARGE_FLOOR..SHARD_COUNT).map(expected_rows).sum::<usize>();
        let ordinary_rows = (0..LARGE_FLOOR).map(expected_rows).sum::<usize>();
        assert_eq!(large_rows, ROW_COUNT / 2);
        assert_eq!(ordinary_rows, ROW_COUNT / 2);
        assert!((0..SHARD_COUNT).all(|shard| expected_rows(shard) == 2));
    }
}
