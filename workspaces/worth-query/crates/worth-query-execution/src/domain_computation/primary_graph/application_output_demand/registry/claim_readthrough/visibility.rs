//! All readers use the eager meaning of a physically deferred edge.
use super::super::refreshed_rejoin::{self, OccurrenceSearchCharge};
use super::super::{DemandRecord, WorthQueryOutputDemandKey};
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct RowObservation {
    pub awaited: bool,
    pub required: bool,
    pub held: bool,
}
fn moving_claims<C: OccurrenceSearchCharge>(
    row: &DemandRecord,
    newest_ready: bool,
    charge: &mut C,
) -> Result<usize, C::Refusal> {
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
            .closed_published(charge)
    } else {
        Ok(0)
    }
}
pub(in super::super) fn observe_row<C: OccurrenceSearchCharge>(
    records: &super::super::record_map::DemandRecords,
    key: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<RowObservation, C::Refusal> {
    let newest = refreshed_rejoin::newest_matching_row(records, key, charge, |_, _| true)?;
    let newest_ready = newest
        .as_ref()
        .and_then(|key| records.get(key))
        .is_some_and(|row| row.has_cached_ready() && !refreshed_rejoin::superseded(row));
    let mut moved_here = false;

    for (_, row) in refreshed_rejoin::occurrence_rows(records, key, charge)? {
        charge.inspect_row()?;
        let moving = moving_claims(row, newest_ready, charge)?;
        moved_here |= newest.as_ref() == Some(key) && moving != 0;
    }
    let awaited =
        refreshed_rejoin::awaited_by_stale_owner_matching(records, key, charge, |row, charge| {
            Ok(row.interests != 0
                || row.framework_required_count > moving_claims(row, newest_ready, charge)?)
        })?;
    let required = if let Some(row) = records.get(key) {
        moved_here
            || row.interests != 0
            || row.required_interests != 0
            || !row.performed_obligations.is_empty()
            || row.framework_required_count > moving_claims(row, newest_ready, charge)?
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
    Ok(RowObservation {
        awaited,
        required,
        held,
    })
}
