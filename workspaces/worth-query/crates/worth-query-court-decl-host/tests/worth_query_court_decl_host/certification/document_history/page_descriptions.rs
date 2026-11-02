//! Compare the weaker page description with an independently retained canonical step.
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryPageEntry, WorthQueryHistoryParentBoundary, WorthQueryHistoryStep,
};
pub(crate) fn assert_page_entry(
    actual: &WorthQueryHistoryPageEntry,
    expected: &WorthQueryHistoryStep,
) {
    assert_eq!(actual.id(), expected.id());
    assert_eq!(actual.label(), expected.label());
    assert_eq!(actual.endpoint(), expected.endpoint());
    assert_eq!(
        actual.parent(),
        expected.parent().map_or(
            WorthQueryHistoryParentBoundary::Root,
            WorthQueryHistoryParentBoundary::Retained
        )
    );
}
pub(crate) fn assert_page_entries(
    actual: &[WorthQueryHistoryPageEntry],
    expected: &[WorthQueryHistoryStep],
) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_page_entry(actual, expected);
    }
}
