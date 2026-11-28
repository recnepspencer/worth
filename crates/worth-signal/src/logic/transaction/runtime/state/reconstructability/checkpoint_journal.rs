use serde::{Deserialize, Serialize};

use crate::data::telemetry::CheckpointTelemetry;
use crate::diagnostics::replay::{ReplayEvent, ReplayEventKind};
use crate::diagnostics::ReplayCursor;
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};
use crate::logic::transaction::TransactionReplayEntry;
use crate::state::{SignalBranchId, SignalSnapshotId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CheckpointRecord {
    pub checkpoint_flushes: u64,
    pub checkpoint_flush_nanos: u128,
    pub rollback_count: u64,
    pub checkpoint_size: u64,
    pub journal_replay_span: u64,
}

impl CheckpointRecord {
    pub fn from_checkpoint_telemetry(telemetry: CheckpointTelemetry) -> Self {
        Self {
            checkpoint_flushes: telemetry.checkpoint_flushes,
            checkpoint_flush_nanos: telemetry.checkpoint_flush_nanos,
            rollback_count: telemetry.rollback_count,
            checkpoint_size: telemetry.checkpoint_size,
            journal_replay_span: telemetry.journal_replay_span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct JournalSegment {
    pub replay_event_count: u32,
    pub first_execution_record_id: Option<ExecutionRecordId>,
    pub last_execution_record_id: Option<ExecutionRecordId>,
    pub first_semantic_segment_id: Option<SemanticSegmentId>,
    pub last_semantic_segment_id: Option<SemanticSegmentId>,
    pub contains_rollback: bool,
    pub contains_failure: bool,
}

impl JournalSegment {
    pub fn from_entries(entries: &[TransactionReplayEntry]) -> Self {
        let mut segment = Self {
            replay_event_count: entries.len() as u32,
            ..Self::default()
        };
        for entry in entries {
            segment.first_execution_record_id = segment
                .first_execution_record_id
                .or(entry.execution_record_id);
            segment.last_execution_record_id = entry
                .execution_record_id
                .or(segment.last_execution_record_id);
            segment.first_semantic_segment_id = segment
                .first_semantic_segment_id
                .or(entry.semantic_segment_id);
            segment.last_semantic_segment_id = entry
                .semantic_segment_id
                .or(segment.last_semantic_segment_id);
            segment.contains_rollback |=
                matches!(entry.kind, ReplayEventKind::TransactionRolledBack);
            segment.contains_failure |= matches!(entry.kind, ReplayEventKind::FailureRecorded);
        }
        segment
    }

    pub fn from_replay_events(entries: &[ReplayEvent]) -> Self {
        let mut segment = Self {
            replay_event_count: entries.len() as u32,
            ..Self::default()
        };
        for entry in entries {
            let execution_record_id = entry
                .execution_record_id
                .map(crate::logic::planner::ExecutionRecordId);
            let semantic_segment_id = entry
                .semantic_segment_id
                .map(crate::logic::planner::SemanticSegmentId);
            segment.first_execution_record_id =
                segment.first_execution_record_id.or(execution_record_id);
            segment.last_execution_record_id =
                execution_record_id.or(segment.last_execution_record_id);
            segment.first_semantic_segment_id =
                segment.first_semantic_segment_id.or(semantic_segment_id);
            segment.last_semantic_segment_id =
                semantic_segment_id.or(segment.last_semantic_segment_id);
            segment.contains_rollback |=
                matches!(entry.kind, ReplayEventKind::TransactionRolledBack);
            segment.contains_failure |= matches!(entry.kind, ReplayEventKind::FailureRecorded);
        }
        segment
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointBoundary {
    pub authority_branch_id: SignalBranchId,
    pub authority_snapshot_id: Option<SignalSnapshotId>,
    pub replay_head: Option<ReplayCursor>,
    pub checkpoint: CheckpointRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedJournalSegment {
    pub replay_head: Option<ReplayCursor>,
    pub replay_event_count: u32,
    pub first_execution_record_id: Option<ExecutionRecordId>,
    pub last_execution_record_id: Option<ExecutionRecordId>,
    pub first_semantic_segment_id: Option<SemanticSegmentId>,
    pub last_semantic_segment_id: Option<SemanticSegmentId>,
    pub contains_rollback: bool,
    pub contains_failure: bool,
}

impl BoundedJournalSegment {
    pub fn from_record(replay_head: Option<ReplayCursor>, segment: &JournalSegment) -> Self {
        Self {
            replay_head,
            replay_event_count: segment.replay_event_count,
            first_execution_record_id: segment.first_execution_record_id,
            last_execution_record_id: segment.last_execution_record_id,
            first_semantic_segment_id: segment.first_semantic_segment_id,
            last_semantic_segment_id: segment.last_semantic_segment_id,
            contains_rollback: segment.contains_rollback,
            contains_failure: segment.contains_failure,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyIndexRebuildProof {
    pub authority_branch_id: SignalBranchId,
    pub authority_snapshot_id: Option<SignalSnapshotId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySuffixRebuildProof {
    pub replay_head: Option<ReplayCursor>,
    pub replay_event_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequiredDerivedRebuildSet {
    DependencyIndexes(DependencyIndexRebuildProof),
    ReplaySuffix(ReplaySuffixRebuildProof),
    TemporalState(TemporalStateRebuildProof),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalStateRebuildProof {
    pub authority_branch_id: SignalBranchId,
    pub authority_snapshot_id: Option<SignalSnapshotId>,
    pub scheduled_wake_count: u64,
    pub ready_wake_count: u64,
    pub retired_wake_count: u64,
}
