//! Discover the request's notes while making its ordinary paid occurrence scan.
use super::super::DemandRegistryState;
use super::super::DemandState;
use super::super::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use worth_runtime_world::facade::CompositeCommitIdentity;
pub(super) struct OccurrenceNotes {
    pub delivery: Option<WorthQueryOutputDemandKey>,
    pub closed: Option<super::super::cleanup::closed_notes::QueuedClosedRow>,
    pub custody: bool,
}
pub(super) fn discover(
    state: &DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,
    selected: Option<&CompositeCommitIdentity>,
    stable_refresh: bool,
) -> Result<OccurrenceNotes, WorthQueryOutputDemandDenial> {
    let mut notes = OccurrenceNotes {
        delivery: None,
        closed: None,
        custody: false,
    };
    for (key, record) in super::super::refreshed_rejoin::occurrence_rows(&state.records, requested)
    {
        if notes.closed.is_none() {
            notes.closed = record.closed_cleanup.queued_note();
        }
        for commit in &record.source_commits {
            notes.custody |= state
                .source_custody
                .get(commit)
                .is_some_and(|entry| entry.cleanup.queued);
        }
        let pending_commit = match &record.state {
            DemandState::Output(output) => output.published_commit.as_ref(),
            _ => None,
        };
        if !stable_refresh
            && key.producer == requested.producer
            && key.applicability == requested.applicability
            && selected.is_some_and(|selected| pending_commit == Some(selected))
        {
            notes.delivery = Some(key.clone());
            break;
        }
    }
    Ok(notes)
}
