use std::sync::atomic::{AtomicUsize, Ordering};

use crate::facade::{RelationalBridgeSourceError, TruthSnapshotIdentity};

use worth_relational::facade::change_source::{
    RelationalCommitSelection, RelationalCommitSelectionDenial, RelationalCommitSelectionWork,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::runtime::RelationalRuntime;

use super::{
    RelationalBridgeSelectedCommitObservation, RelationalBridgeSelectedObservation,
    RuntimeBridgeRelationalSource,
};

/// The commit selections one source and its clones have made, and the
/// Relational ancestry work they did, summed over every publication path.
#[derive(Debug, Default)]
pub(super) struct SourceSelectionWork {
    selections: AtomicUsize,
    ancestry_visits: AtomicUsize,
}

impl SourceSelectionWork {
    fn record(&self, work: RelationalCommitSelectionWork) {
        self.selections
            .fetch_add(work.selections(), Ordering::Relaxed);
        self.ancestry_visits
            .fetch_add(work.ancestry_visits(), Ordering::Relaxed);
    }
}

impl RuntimeBridgeRelationalSource {
    /// Commit selections this source and its clones have made so far, and the
    /// ancestry visits those selections did.
    #[cfg(test)]
    pub(in crate::relational_source) fn selection_work_totals(&self) -> (usize, usize) {
        (
            self.selection_work.selections.load(Ordering::Relaxed),
            self.selection_work.ancestry_visits.load(Ordering::Relaxed),
        )
    }
}

impl RelationalBridgeSelectedObservation {
    pub(super) fn select_reachable_commit(
        self,
        runtime: &RelationalRuntime,
        commit_id: CommitId,
        work: &SourceSelectionWork,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        let selection = runtime.select_reachable_commit(&self.observation, commit_id);
        self.into_source_selection(selection, work, "has no committed selected root")
    }

    pub(super) fn select_exact_selected_commit(
        self,
        runtime: &RelationalRuntime,
        commit_id: CommitId,
        work: &SourceSelectionWork,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        let selection = runtime.select_exact_commit(&self.observation, commit_id);
        self.into_source_selection(selection, work, "has no selected commit")
    }

    fn into_source_selection(
        self,
        selection: RelationalCommitSelection,
        work: &SourceSelectionWork,
        no_commit_detail: &str,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        work.record(selection.work());
        let snapshot_identity = self.snapshot_identity;
        match selection.into_outcome() {
            Ok(selected) => Ok(RelationalBridgeSelectedCommitObservation {
                selected,
                snapshot_identity,
            }),
            Err(denial) => Err(selection_error(
                &snapshot_identity,
                denial,
                no_commit_detail,
            )),
        }
    }
}

fn selection_error(
    snapshot: &TruthSnapshotIdentity,
    denial: RelationalCommitSelectionDenial,
    no_commit_detail: &str,
) -> RelationalBridgeSourceError {
    RelationalBridgeSourceError::new(crate::adapter::RelationalBridgeSourceErrorTag::CommitSelection(denial), match denial {
        RelationalCommitSelectionDenial::ForeignObservation => {
            format!("relational bridge snapshot {snapshot:?} was observed in another runtime")
        }
        RelationalCommitSelectionDenial::NoSelectedCommit => {
            format!("relational bridge snapshot {snapshot:?} {no_commit_detail}")
        }
        RelationalCommitSelectionDenial::SelectedCommitUnavailable { selected } => format!(
            "relational bridge snapshot {snapshot:?} selects unavailable commit `{}`",
            selected.0
        ),
        RelationalCommitSelectionDenial::RequestedCommitUnavailable { requested } => format!(
            "relational bridge snapshot {snapshot:?} cannot see unavailable requested commit `{}`",
            requested.0
        ),
        RelationalCommitSelectionDenial::Unreachable {
            selected,
            requested,
        } => format!(
            "relational bridge snapshot {snapshot:?} at commit `{}` cannot see requested commit `{}` without an exact retained historical observation",
            selected.0, requested.0,
        ),
        RelationalCommitSelectionDenial::NotSelectedCommit {
            selected,
            requested,
        } => format!(
            "relational bridge snapshot {snapshot:?} selects commit `{}` rather than exact requested commit `{}`",
            selected.0, requested.0
        ),
    })
}
