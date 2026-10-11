use super::*;
#[test]
fn all_demand_key_sites_share_declared_comparison_widths() {
    let (short, long) = fixtures::keys();
    let one = comparison_work([&short]).unwrap();
    assert_eq!((one.maximum, one.total), (4 + 1 + 1 + 10, 4 + 1 + 1 + 10));
    let pair = comparison_work([&short, &long]).unwrap();
    assert_eq!(pair.maximum, 4 + 15 + 16 + 10);
    assert_eq!(pair.total, (4 + 1 + 1 + 10) + (4 + 15 + 16 + 10));
}
#[test]
fn empty_predecessor_selection_compares_no_predecessor_keys() {
    // Checkpoint inputs can exist without any selected in-memory predecessor.
    // Empty sort/dedup, lookup and rollback visit no predecessor keys. The
    // downstream record lookup is separately priced as a one-key comparison.
    let work = comparison_work(std::iter::empty()).unwrap();
    assert_eq!((work.maximum, work.total), (0, 0));
    assert_eq!(work.sort_bound(3), Some(0));
    assert_eq!(
        super::super::prerequisite_work::prior_release_work(12, 0, work.maximum).unwrap(),
        0
    );
}
