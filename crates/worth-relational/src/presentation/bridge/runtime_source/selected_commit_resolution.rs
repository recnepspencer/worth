use worth_runtime_bridge::facade::{RelationalBridgeSourceError, TruthSnapshotIdentity};

use crate::facade::change_source::{
    RelationalCommitSelection, RelationalCommitSelectionDenial, RelationalCommitSelectionWork,
};
use crate::facade::history::CommitId;
use crate::facade::runtime::RelationalRuntime;

use super::{RelationalBridgeSelectedCommitObservation, RelationalBridgeSelectedObservation};

/// One adapter commit selection and the Relational work it did.
pub(in crate::presentation::bridge) struct SourceCommitSelection {
    outcome: Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError>,
    /// Read by the adapter's cost tests. Production callers need only the
    /// outcome; the runtime's own counters record the same work.
    #[cfg_attr(not(test), allow(dead_code))]
    work: RelationalCommitSelectionWork,
}

impl SourceCommitSelection {
    #[cfg(test)]
    pub(in crate::presentation::bridge) fn work(&self) -> RelationalCommitSelectionWork {
        self.work
    }

    pub(in crate::presentation::bridge) fn into_result(
        self,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        self.outcome
    }
}

impl RelationalBridgeSelectedObservation {
    pub(super) fn select_reachable_commit(
        self,
        runtime: &RelationalRuntime,
        commit_id: CommitId,
    ) -> SourceCommitSelection {
        let selection = runtime.select_reachable_commit(&self.observation, commit_id);
        self.into_source_selection(selection, "has no committed selected root")
    }

    pub(super) fn select_exact_selected_commit(
        self,
        runtime: &RelationalRuntime,
        commit_id: CommitId,
    ) -> SourceCommitSelection {
        let selection = runtime.select_exact_commit(&self.observation, commit_id);
        self.into_source_selection(selection, "has no selected commit")
    }

    fn into_source_selection(
        self,
        selection: RelationalCommitSelection,
        no_commit_detail: &str,
    ) -> SourceCommitSelection {
        let work = selection.work();
        let snapshot_identity = self.snapshot_identity;
        let outcome = match selection.into_outcome() {
            Ok(selected) => Ok(RelationalBridgeSelectedCommitObservation {
                selected,
                snapshot_identity,
            }),
            Err(denial) => Err(selection_error(&snapshot_identity, denial, no_commit_detail)),
        };
        SourceCommitSelection { outcome, work }
    }
}

fn selection_error(
    snapshot: &TruthSnapshotIdentity,
    denial: RelationalCommitSelectionDenial,
    no_commit_detail: &str,
) -> RelationalBridgeSourceError {
    RelationalBridgeSourceError::new(match denial {
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
