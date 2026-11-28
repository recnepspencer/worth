mod retained_charge;
mod work_admission;

mod artifact_records;
mod branch_records;
mod observations;
mod snapshot_records;

use serde::{Deserialize, Serialize};

use super::identity::LineageArtifactId;
use super::transitions::{ArtifactTransitionKind, InvalidationCause, SnapshotRestoreKind};
use crate::data::handle::NodeId;
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};

use crate::state::{SignalBranchId, SignalSnapshotId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageRecordKind {
    ArtifactTransition {
        node: NodeId,
        artifact_id: LineageArtifactId,
        parent_artifact_id: Option<LineageArtifactId>,
        execution_record_id: ExecutionRecordId,
        semantic_segment_id: SemanticSegmentId,
        transition: ArtifactTransitionKind,
    },
    BranchFork {
        created_branch_id: SignalBranchId,
        parent_branch_id: SignalBranchId,
        created_branch_display_name: String,
        parent_branch_display_name: String,
    },
    BranchSwitch {
        from_branch_id: SignalBranchId,
        to_branch_id: SignalBranchId,
        from_branch_display_name: String,
        to_branch_display_name: String,
    },

    SnapshotRestore {
        snapshot_id: SignalSnapshotId,
        node: Option<NodeId>,
        artifact_id: Option<LineageArtifactId>,
        restore_kind: SnapshotRestoreKind,
    },
    Invalidation {
        node: NodeId,
        artifact_id: LineageArtifactId,
        cause: InvalidationCause,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageRecord {
    /// Monotonic lineage event order within a live runtime history.
    ///
    /// This is a runtime-local ordering clock used for canonical lineage
    /// ordering and replay/history stitching. It must stay unique and
    /// increasing within a live branched runtime, but it is not a claim about
    /// wall-clock order outside that runtime.
    pub sequence: u64,
    /// Branch context on which the lineage record was emitted.
    ///
    /// For branch-topology records, this is the active branch timeline that
    /// observed/emitted the event, not necessarily the same as the branch ids
    /// named inside the record kind payload.
    pub emitted_on_branch_id: SignalBranchId,
    pub kind: LineageRecordKind,
}

impl LineageRecord {
    pub fn new(
        sequence: u64,
        emitted_on_branch_id: SignalBranchId,
        kind: LineageRecordKind,
    ) -> Self {
        Self {
            sequence,
            emitted_on_branch_id,
            kind,
        }
    }
}
