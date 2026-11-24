//! Original revisions across an actual committed graph selection; never write authority.
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::history::data::CanonicalCommitAuthorityKind;
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::mvcc::observation::RelationalBranchObservation;
use crate::visibility::materialization::read_records::projection::{
    RelationalBorrowedRecordReadDenial, VisibilityProjectionView,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
pub(crate) mod antecedent;
mod chain;
mod output_routes;
mod revisions;
pub(crate) mod source_presence;
pub use source_presence::{
    RelationalHistoricalSourcePresence, RelationalHistoricalSourceReadDenial,
};

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
    routes: std::sync::OnceLock<output_routes::SelectedOutputRouteIndex>,
    _memory: Option<ExecutionMemoryReservation>,
}

/// One independently authenticated committed selection in oldest-first order.
struct HistoricalSelectionStage {
    selected: VisibilityProjectionView,
    restored: VisibilityProjectionView,
    envelope: Arc<CanonicalCommitEnvelope>,
    _memory: Option<ExecutionMemoryReservation>,
}

#[path = "historical_selection/runtime_reads.rs"]
mod runtime_reads;

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
