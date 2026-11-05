//! Original revisions across an actual committed graph selection; never write authority.
use super::super::reader::VisibilityReadContext;
use super::{RelationalBorrowedRecordReadDenial, VisibilityProjectionView};
use crate::{
    execution::RelationalExecutionDenialCause as Cause,
    history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, CommitId},
    mvcc::RelationalBranchObservation,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
mod antecedent;
mod chain;
mod output_routes;
mod revisions;
mod source_presence;
pub use source_presence::RelationalHistoricalSourcePresence;

#[derive(Debug)]
pub enum RelationalHistoricalSelectionReadDenial {
    ForeignOwner,
    NotGraphSelection { commit: Option<CommitId> },
    SelectedBasisMismatch { commit: CommitId },
    Execution(Cause),
    Request(worth_execution::WorkCeilingDenial),
}

/// Retains the actual selected and freshly restored Native roots. Original
/// producer facts remain immutable; only members named by the committed
/// selection can compare their original revisions through this read evidence.
/// There is no constructor from a descriptor, stamp, or caller member list.
pub struct RelationalHistoricalSelectionRead {
    stages: Vec<HistoricalSelectionStage>,
    _memory: Option<ExecutionMemoryReservation>,
}

/// One independently authenticated committed selection in oldest-first order.
struct HistoricalSelectionStage {
    selected: VisibilityProjectionView,
    restored: VisibilityProjectionView,
    envelope: Arc<CanonicalCommitEnvelope>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl VisibilityReadContext<'_> {
    pub fn read_historical_selection(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        selected: &RelationalBranchObservation,
        restored: &RelationalBranchObservation,
    ) -> Result<RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        crate::execution::check_request(execution).map_err(Denial::Execution)?;
        for source in [selected, restored] {
            if source.identity().runtime_instance_id() != self.runtime().runtime_instance_id() {
                return Err(Denial::ForeignOwner);
            }
        }
        let commit = restored
            .selected_root()
            .commit_id()
            .ok_or(Denial::NotGraphSelection { commit: None })?;
        let envelope = self
            .runtime()
            .history()
            .demand_commit_envelope(execution, commit)
            .map_err(Denial::Execution)?
            .ok_or(Denial::NotGraphSelection {
                commit: Some(commit),
            })?;
        let CanonicalCommitAuthorityKind::OwnedGraphSelection(graph) = &envelope.authority_kind
        else {
            return Err(Denial::NotGraphSelection {
                commit: Some(commit),
            });
        };
        // Canonicalization copies branch text and parent IDs, never records.
        // Fund both its temporary backing and this retained read capsule first.
        let descriptor = selected.descriptor();
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let bytes = (std::mem::size_of::<RelationalHistoricalSelectionRead>() as u64)
            .checked_add((descriptor.branch_id().0.len() as u64).saturating_mul(4))
            .and_then(|n| n.checked_add((parents as u64).saturating_mul(32)))
            .and_then(|n| n.checked_add(1024))
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, bytes)
            })
            .map_err(Denial::Request)?
            .0
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let canonical = descriptor
            .in_canonical_frame()
            .map_err(|_| Denial::SelectedBasisMismatch { commit })?;
        if graph.selected_basis() != &canonical
            || selected.descriptor().schema_commitment()
                != restored.descriptor().schema_commitment()
            || envelope.commit.version_id != restored.version_id()
            || restored.commit_id() != Some(commit)
        {
            return Err(Denial::SelectedBasisMismatch { commit });
        }
        let project = |source| {
            VisibilityProjectionView::new(
                self.runtime(),
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
                    crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
                        source,
                    ),
                ),
            )
        };
        Ok(RelationalHistoricalSelectionRead {
            stages: vec![HistoricalSelectionStage {
                selected: project(selected),
                restored: project(restored),
                envelope,
                _memory: memory,
            }],
            _memory: None,
        })
    }
}

impl RelationalHistoricalSelectionRead {
    fn latest(&self) -> &HistoricalSelectionStage {
        self.stages
            .last()
            .expect("read evidence has a committed selection")
    }
    pub fn model(&self) -> crate::identity::data::EntityId {
        self.latest().graph().model()
    }
    pub fn selection_commit(&self) -> CommitId {
        self.latest().envelope.commit.commit_id
    }
}

impl HistoricalSelectionStage {
    fn graph(&self) -> &crate::history::data::CanonicalOwnedGraphSelection {
        match &self.envelope.authority_kind {
            CanonicalCommitAuthorityKind::OwnedGraphSelection(graph) => graph,
            _ => unreachable!("only the native graph-selection reader constructs this evidence"),
        }
    }
}

fn borrowed(cause: RelationalBorrowedRecordReadDenial) -> RelationalHistoricalSelectionReadDenial {
    match cause {
        RelationalBorrowedRecordReadDenial::Execution(cause) => {
            RelationalHistoricalSelectionReadDenial::Execution(cause)
        }
        RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
            RelationalHistoricalSelectionReadDenial::ForeignOwner
        }
    }
}
