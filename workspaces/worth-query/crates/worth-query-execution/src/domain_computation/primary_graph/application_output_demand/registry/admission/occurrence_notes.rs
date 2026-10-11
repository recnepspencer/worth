//! Discover the request's notes while making its ordinary paid occurrence scan.
use super::super::{DemandRegistryState, DemandState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, WorthQueryOutputDemandDenial,
};
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
    admission: &mut InvalidationEditAdmission,
) -> Result<OccurrenceNotes, WorthQueryOutputDemandDenial> {
    let mut notes = OccurrenceNotes {
        delivery: None,
        closed: None,
        custody: false,
    };
    for (key, record) in
        super::super::refreshed_rejoin::occurrence_rows(&state.records, requested, admission)?
    {
        admission
            .charge_external_work(1)
            .map_err(|_| super::super::refreshed_rejoin::work_denial())?;
        if notes.closed.is_none() {
            notes.closed = record.closed_cleanup.queued_note();
        }
        for commit in &record.source_commits {
            admission
                .charge_external_work(1)
                .map_err(|_| super::super::refreshed_rejoin::work_denial())?;
            notes.custody |= state
                .source_custody
                .get(commit)
                .is_some_and(|entry| entry.cleanup.queued);
        }
        let pending_commit = match &record.state {
            DemandState::Output(output) => output.published_commit.as_ref(),
            _ => None,
        };
        if !stable_refresh && selected.is_some_and(|selected| pending_commit == Some(selected)) {
            notes.delivery = Some(key.clone());
            break;
        }
    }
    Ok(notes)
}
