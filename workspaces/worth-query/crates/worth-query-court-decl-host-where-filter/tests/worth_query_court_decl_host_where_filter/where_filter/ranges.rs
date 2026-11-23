//! Journeys 2 and 3: parameterized Date and numeric range filters.
use super::*;

#[test]
fn one_query_combines_independently_optional_date_and_text() {
    let host = install(&board());
    let due = |max_due_days, text: Option<&str>| {
        keys(world::due_by(
            &host,
            DueBy {
                project: 1,
                max_due_days,
                text: text.map(str::to_owned),
            },
            None,
        ))
    };
    // Due days: 1→100, 2→absent, 3→50, 4→200, 5→150.
    assert_eq!(due(Some(150), None), [1, 3, 5]);
    assert_eq!(due(Some(50), None), [3]);
    assert!(due(Some(49), None).is_empty());
    assert_eq!(due(Some(10_000), None), [1, 3, 4, 5]);
    assert_eq!(due(None, None), [1, 2, 3, 4, 5]);
    assert_eq!(due(None, Some("fLaNgE")), [1, 2, 5]);
    assert_eq!(due(Some(150), Some("fLaNgE")), [1, 5]);
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
