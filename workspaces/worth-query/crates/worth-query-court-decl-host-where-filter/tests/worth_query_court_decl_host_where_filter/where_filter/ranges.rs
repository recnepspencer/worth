//! Journeys 2 and 3: parameterized Date and numeric range filters.
use super::*;

fn due(max_due_days: i64) -> Vec<u64> {
    keys(world::due_by(
        &install(&board()),
        DueBy {
            project: 1,
            max_due_days,
        },
        None,
    ))
}

#[test]
fn date_range_is_inclusive_and_an_absent_due_date_never_matches() {
    // Due days: 1→100, 2→absent, 3→50, 4→200, 5→150.
    assert_eq!(due(150), [1, 3, 5]);
    assert_eq!(due(50), [3]);
    assert!(due(49).is_empty());
    assert_eq!(due(10_000), [1, 3, 4, 5]);
}

#[test]
fn numeric_part_fit_combines_an_integer_floor_and_a_float_ceiling() {
    let host = install(&board());
    let fit = |min_rank, max_length_micros| {
        keys(world::fit(
            &host,
            Fit {
                project: 1,
                min_rank,
                max_length_micros,
            },
            None,
        ))
    };
    // (rank, length): 1→(5, 12.5) 2→(9, 40.0) 3→(7, 3.0) 4→(2, 80.0) 5→(9, 25.0)
    assert_eq!(fit(7, 30_000_000), [3, 5]);
    assert_eq!(fit(9, 40_000_000), [2, 5]);
    assert_eq!(fit(0, 12_500_000), [1, 3]);
    assert!(fit(10, 100_000_000).is_empty());
}
