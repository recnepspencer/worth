//! All readers use the eager meaning of a physically deferred edge.
use super::super::refreshed_rejoin;
use super::super::DemandRecord;
use super::super::WorthQueryOutputDemandKey;
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct RowObservation {
    pub awaited: bool,
    pub required: bool,
    pub held: bool,
}
fn moving_claims(row: &DemandRecord, newest_ready: bool) -> usize {
    if newest_ready
        && row.interests == 0
        && row.required_interests == 0
        && refreshed_rejoin::superseded(row)
    {
        row.closed_cleanup
            .node()
            .claimants
            .lock()
            .unwrap()
            .closed_published()
    } else {
        0
    }
}
pub(in super::super) fn observe_row(
    records: &super::super::record_map::DemandRecords,
    key: &WorthQueryOutputDemandKey,
) -> RowObservation {
    let newest = refreshed_rejoin::newest_matching_row(records, key, |_, _| true);
    let newest_ready = newest
        .as_ref()
        .and_then(|key| records.get(key))
        .is_some_and(|row| row.has_cached_ready() && !refreshed_rejoin::superseded(row));
    let mut moved_here = false;

    for (_, row) in refreshed_rejoin::occurrence_rows(records, key) {
        let moving = moving_claims(row, newest_ready);
        moved_here |= newest.as_ref() == Some(key) && moving != 0;
    }
    let awaited = refreshed_rejoin::awaited_by_stale_owner_matching(records, key, |row| {
        row.interests != 0 || row.framework_required_count > moving_claims(row, newest_ready)
    });
    let required = if let Some(row) = records.get(key) {
        moved_here
            || row.interests != 0
            || row.required_interests != 0
            || !row.performed_obligations.is_empty()
            || row.framework_required_count > moving_claims(row, newest_ready)
    } else {
        moved_here
    };
    let held = required
        || awaited
        || records.get(key).is_some_and(|row| {
            row.prepared_prerequisite_claims != 0
                || row.performed_source.is_some()
                || row.held_successor.is_some()
        });
    RowObservation {
        awaited,
        required,
        held,
    }
}
