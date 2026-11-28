use super::{
    ArtifactTransitionKind, InvalidationCause, LineageArtifactId, LineageRecord, LineageRecordKind,
};
use crate::data::handle::NodeId;
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};
use crate::state::{SignalBranchId, SignalSnapshotId};

impl LineageRecord {
    pub fn node(&self) -> Option<NodeId> {
        match &self.kind {
            LineageRecordKind::ArtifactTransition { node, .. }
            | LineageRecordKind::Invalidation { node, .. } => Some(*node),

            LineageRecordKind::SnapshotRestore { node, .. } => *node,
            LineageRecordKind::BranchFork { .. } | LineageRecordKind::BranchSwitch { .. } => None,
        }
    }

    pub fn emitted_on_branch_id(&self) -> SignalBranchId {
        self.emitted_on_branch_id
    }

    pub fn subject_artifact_id(&self) -> Option<LineageArtifactId> {
        match &self.kind {
            LineageRecordKind::ArtifactTransition { artifact_id, .. }
            | LineageRecordKind::Invalidation { artifact_id, .. } => Some(*artifact_id),
            LineageRecordKind::SnapshotRestore { artifact_id, .. } => *artifact_id,
            LineageRecordKind::BranchFork { .. } | LineageRecordKind::BranchSwitch { .. } => None,
        }
    }

    pub fn parent_artifact_id(&self) -> Option<LineageArtifactId> {
        match &self.kind {
            LineageRecordKind::ArtifactTransition {
                parent_artifact_id, ..
            } => *parent_artifact_id,
            LineageRecordKind::SnapshotRestore { .. }
            | LineageRecordKind::Invalidation { .. }
            | LineageRecordKind::BranchFork { .. }
            | LineageRecordKind::BranchSwitch { .. } => None,
        }
    }

    pub fn restored_artifact_id(&self) -> Option<LineageArtifactId> {
        match &self.kind {
            LineageRecordKind::SnapshotRestore { artifact_id, .. } => *artifact_id,
            _ => None,
        }
    }

    pub fn invalidated_artifact_id(&self) -> Option<LineageArtifactId> {
        match &self.kind {
            LineageRecordKind::Invalidation { artifact_id, .. } => Some(*artifact_id),
            _ => None,
        }
    }

    pub fn invalidation_cause(&self) -> Option<&InvalidationCause> {
        match &self.kind {
            LineageRecordKind::Invalidation { cause, .. } => Some(cause),
            _ => None,
        }
    }

    pub fn execution_record_id(&self) -> Option<ExecutionRecordId> {
        match &self.kind {
            LineageRecordKind::ArtifactTransition {
                execution_record_id,
                ..
            } => Some(*execution_record_id),
            _ => None,
        }
    }

    pub fn semantic_segment_id(&self) -> Option<SemanticSegmentId> {
        match &self.kind {
            LineageRecordKind::ArtifactTransition {
                semantic_segment_id,
                ..
            } => Some(*semantic_segment_id),
            _ => None,
        }
    }

    pub fn snapshot_id(&self) -> Option<SignalSnapshotId> {
        match &self.kind {
            LineageRecordKind::SnapshotRestore { snapshot_id, .. } => Some(*snapshot_id),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.kind {
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::Replaced,
                ..
            } => "Replaced",
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::Refreshed { .. },
                ..
            } => "Refreshed",
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::MemoizedReuse,
                ..
            } => "MemoizedReuse",
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::SnapshotRestoreReuse,
                ..
            } => "SnapshotRestoreReuse",
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::ReconciliationAdoption,
                ..
            } => "ReconciliationAdoption",
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::CrossIdentityPersistentReuse { .. },
                ..
            } => "CrossIdentityPersistentReuse",
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::PartialArtifactSplice { .. },
                ..
            } => "PartialArtifactSplice",
            LineageRecordKind::BranchFork { .. } => "BranchedFrom",
            LineageRecordKind::BranchSwitch { .. } => "BranchSwitched",
            LineageRecordKind::SnapshotRestore { .. } => "Restored",
            LineageRecordKind::Invalidation { .. } => "InvalidatedWithoutReplacement",
        }
    }
}
