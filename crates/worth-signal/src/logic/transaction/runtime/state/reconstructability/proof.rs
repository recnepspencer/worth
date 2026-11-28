use serde::{Deserialize, Serialize};

use crate::diagnostics::ReplayCursor;
use crate::logic::transaction::TransactionReplayEntry;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::{
    BoundedJournalSegment, CheckpointBoundary, CheckpointRecord, JournalSegment,
    RequiredDerivedRebuildSet, TemporalReconstructabilityArtifact, TemporalStateRebuildProof,
};
use crate::diagnostics::replay::ReplayEvent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructabilityProof {
    pub checkpoint: CheckpointBoundary,
    pub journal: BoundedJournalSegment,
    pub temporal: TemporalReconstructabilityArtifact,
    pub required_rebuild: Vec<RequiredDerivedRebuildSet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructabilityRecord {
    pub authority_branch_id: SignalBranchId,
    pub authority_snapshot_id: Option<SignalSnapshotId>,
    pub replay_head: Option<ReplayCursor>,
    pub checkpoint: CheckpointRecord,
    pub journal: JournalSegment,
    #[serde(default)]
    pub temporal: TemporalReconstructabilityArtifact,
}

impl ReconstructabilityRecord {
    pub fn from_transaction_boundary(
        authority_branch_id: SignalBranchId,
        authority_snapshot_id: Option<SignalSnapshotId>,
        replay_head: Option<ReplayCursor>,
        checkpoint: CheckpointRecord,
        replay_entries: &[TransactionReplayEntry],
        temporal: TemporalReconstructabilityArtifact,
    ) -> Self {
        Self {
            authority_branch_id,
            authority_snapshot_id,
            replay_head,
            checkpoint,
            journal: JournalSegment::from_entries(replay_entries),
            temporal,
        }
    }

    pub fn from_snapshot_boundary(
        authority_branch_id: SignalBranchId,
        authority_snapshot_id: SignalSnapshotId,
        replay_head: Option<ReplayCursor>,
        mut checkpoint: CheckpointRecord,
        replay_entries: &[ReplayEvent],
        temporal: TemporalReconstructabilityArtifact,
    ) -> Self {
        let journal = JournalSegment::from_replay_events(replay_entries);
        checkpoint.journal_replay_span = journal.replay_event_count as u64;
        Self {
            authority_branch_id,
            authority_snapshot_id: Some(authority_snapshot_id),
            replay_head,
            checkpoint,
            journal,
            temporal,
        }
    }

    pub fn checkpoint_boundary(&self) -> CheckpointBoundary {
        CheckpointBoundary {
            authority_branch_id: self.authority_branch_id,
            authority_snapshot_id: self.authority_snapshot_id,
            replay_head: self.replay_head,
            checkpoint: self.checkpoint,
        }
    }

    pub fn required_derived_rebuild_set(&self) -> Vec<RequiredDerivedRebuildSet> {
        let mut rebuild = vec![RequiredDerivedRebuildSet::DependencyIndexes(
            super::DependencyIndexRebuildProof {
                authority_branch_id: self.authority_branch_id,
                authority_snapshot_id: self.authority_snapshot_id,
            },
        )];
        rebuild.push(RequiredDerivedRebuildSet::ReplaySuffix(
            super::ReplaySuffixRebuildProof {
                replay_head: self.replay_head,
                replay_event_count: self.journal.replay_event_count,
            },
        ));
        rebuild.push(RequiredDerivedRebuildSet::TemporalState(
            TemporalStateRebuildProof {
                authority_branch_id: self.authority_branch_id,
                authority_snapshot_id: self.authority_snapshot_id,
                scheduled_wake_count: self.temporal.wake_summary.scheduled_count(),
                ready_wake_count: self.temporal.wake_summary.ready_count(),
                retired_wake_count: self.temporal.wake_summary.retired_count(),
            },
        ));
        rebuild
    }

    pub fn proof(&self) -> ReconstructabilityProof {
        ReconstructabilityProof {
            checkpoint: self.checkpoint_boundary(),
            journal: BoundedJournalSegment::from_record(self.replay_head, &self.journal),
            temporal: self.temporal.clone(),
            required_rebuild: self.required_derived_rebuild_set(),
        }
    }
}
