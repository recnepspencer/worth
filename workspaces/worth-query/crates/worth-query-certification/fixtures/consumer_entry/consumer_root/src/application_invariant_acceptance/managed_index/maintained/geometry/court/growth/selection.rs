//! Checked forward traversal on the genuine world, with the whole read invoice.
use super::{intersecting, invoice, provider, EdgeValue};
use std::sync::atomic::Ordering;

pub(super) fn verify<R>(
    calls: &provider::Calls,
    universe: &[EdgeValue],
    x: i64,
    stage: &str,
    read: impl FnOnce() -> R,
    values: impl FnOnce(&R) -> Vec<EdgeValue>,
) {
    // Oracle interpretation and population counting are outside the read invoice.
    let expected = intersecting(universe.to_vec(), x);
    let count = universe.len();
    let levels = u64::from(usize::BITS - count.leading_zeros());
    let maximum = 4 * levels + 3;
    let comparisons = calls.summary_comparisons.load(Ordering::SeqCst);
    let matches = calls.matched_sources.load(Ordering::SeqCst);
    let (actual, work) = invoice(read);
    let comparisons = calls.summary_comparisons.load(Ordering::SeqCst) - comparisons;
    let matches = calls.matched_sources.load(Ordering::SeqCst) - matches;
    assert_eq!(
        values(&actual),
        expected,
        "{stage} authored membership at {x}"
    );
    assert_eq!(
        matches,
        expected.len(),
        "only surviving genuine leaves match"
    );
    assert!(
        comparisons > 0 && comparisons as u64 <= maximum,
        "{stage} Source={count} x={x}: comparisons={comparisons}, maximum={maximum}"
    );
    assert!(work > 0, "the original read has a genuine SDK invoice");
    eprintln!(
        "checked query {stage} Source={count} x={x} comparisons={comparisons} matches={matches} original SDK invoice={work}"
    );
}
