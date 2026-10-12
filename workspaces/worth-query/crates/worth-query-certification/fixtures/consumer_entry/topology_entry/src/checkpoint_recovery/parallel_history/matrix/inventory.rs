//! The product includes unavailable capabilities as named rows, never skips.
#[test]
fn matrix_rows_name_the_unplaceable_deadline_and_the_separate_placement_proof() {
    let mut rows = Vec::new();
    for family in super::FAMILY_NAMES {
        for workers in [1, 2, 5] {
            rows.push((format!("{family:?}/width-{workers}/in-flight-deadline"),"deadline and cancellation use the same meter safe point; placing a deadline requires a clock control, and none is added"));
        }
    }
    rows.push((
        "cross-placement incremental prior".to_owned(),
        "7.6 part two owns the cross-placement prior proof; this matrix varies leased widths",
    ));
    assert_eq!(
        rows.len(),
        8 * 3 + 1,
        "named not-run rows have their declared product"
    );
    for (name, reason) in rows {
        assert!(!reason.is_empty());
        println!("NOT_RUN:{name}:{reason}");
    }
}
