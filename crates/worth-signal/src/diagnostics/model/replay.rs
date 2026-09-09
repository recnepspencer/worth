mod retained_charge;

use crate::diagnostics::state::DiagnosticHistory;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;
use crate::data::reuse::{PersistentCorrespondenceKind, ReuseOrigin};
use crate::diagnostics::lineage::LineageArtifactId;
use crate::logic::planner::TaskExecutionOutcome;
use crate::logic::transaction::{
    ScopedMergeProofPacket, SignalMergeCompatibilityWitness, SignalMergeStrategyWitness,
};
use crate::state::{SignalBranchId, SignalSnapshotId};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct ReplayCursor(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReplayEventKind {
    TaskApplied,
    TransactionCommitted,
    TransactionRolledBack,
    FailureRecorded,
    SnapshotCaptured,
    SnapshotRestored,
    BranchCreated,
    BranchSwitched,
    BranchMerged,
    BranchRetired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayEventDetail {
    TaskOutcome(TaskExecutionOutcome),
    Message(String),
    BranchMergeSummary {
        message: String,
        #[serde(with = "super::replay_strategy_witness_serde")]
        strategy_witness: SignalMergeStrategyWitness,
        #[serde(with = "super::replay_compatibility_witness_serde")]
        compatibility_witness: SignalMergeCompatibilityWitness,
        scoped_merge_proof: ScopedMergeProofPacket,
    },
}

impl ReplayEventDetail {
    pub fn as_message(&self) -> Option<&str> {
        match self {
            Self::Message(message) | Self::BranchMergeSummary { message, .. } => {
                Some(message.as_str())
            }
            Self::TaskOutcome(_) => None,
        }
    }

    pub fn as_scoped_merge_proof(&self) -> Option<&ScopedMergeProofPacket> {
        match self {
            Self::BranchMergeSummary {
                scoped_merge_proof, ..
            } => Some(scoped_merge_proof),
            Self::TaskOutcome(_) | Self::Message(_) => None,
        }
    }

    pub fn as_strategy_witness(&self) -> Option<&SignalMergeStrategyWitness> {
        match self {
            Self::BranchMergeSummary {
                strategy_witness, ..
            } => Some(strategy_witness),
            Self::TaskOutcome(_) | Self::Message(_) => None,
        }
    }

    pub fn as_compatibility_witness(&self) -> Option<&SignalMergeCompatibilityWitness> {
        match self {
            Self::BranchMergeSummary {
                compatibility_witness,
                ..
            } => Some(compatibility_witness),
            Self::TaskOutcome(_) | Self::Message(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub cursor: ReplayCursor,
    pub kind: ReplayEventKind,
    pub branch_id: SignalBranchId,
    pub snapshot_id: Option<SignalSnapshotId>,
    pub node: Option<NodeId>,
    pub execution_record_id: Option<u64>,
    pub semantic_segment_id: Option<u64>,
    pub lineage_artifact_id: Option<LineageArtifactId>,
    pub reuse_origin: Option<ReuseOrigin>,
    pub persistent_correspondence_kind: Option<PersistentCorrespondenceKind>,
    pub composition_region_count: Option<u32>,
    pub detail: Option<ReplayEventDetail>,
}

impl ReplayEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cursor: ReplayCursor,
        kind: ReplayEventKind,
        branch_id: SignalBranchId,
        snapshot_id: Option<SignalSnapshotId>,
        node: Option<NodeId>,
        execution_record_id: Option<u64>,
        semantic_segment_id: Option<u64>,
        lineage_artifact_id: Option<LineageArtifactId>,
        reuse_origin: Option<ReuseOrigin>,
        persistent_correspondence_kind: Option<PersistentCorrespondenceKind>,
        composition_region_count: Option<u32>,
        detail: Option<ReplayEventDetail>,
    ) -> Self {
        Self {
            cursor,
            kind,
            branch_id,
            snapshot_id,
            node,
            execution_record_id,
            semantic_segment_id,
            lineage_artifact_id,
            reuse_origin,
            persistent_correspondence_kind,
            composition_region_count,
            detail,
        }
    }
}

pub type ReplayFrame = ReplayEvent;

#[derive(Debug, Clone, Copy, Default)]
pub struct RetainedReplayView<'a> {
    start: Option<ReplayCursor>,
    end: Option<ReplayCursor>,
    frames: Option<&'a DiagnosticHistory<ReplayEvent>>,
    offset: usize,
    len: usize,
}

impl<'a> RetainedReplayView<'a> {
    pub(crate) fn new(
        start: Option<ReplayCursor>,
        end: Option<ReplayCursor>,
        frames: &'a DiagnosticHistory<ReplayEvent>,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            start,
            end,
            frames: Some(frames),
            offset,
            len,
        }
    }

    pub fn empty() -> Self {
        Self {
            start: None,
            end: None,
            frames: None,
            offset: 0,
            len: 0,
        }
    }

    pub fn start(&self) -> Option<ReplayCursor> {
        self.start
    }

    pub fn end(&self) -> Option<ReplayCursor> {
        self.end
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &'a ReplayEvent> + ExactSizeIterator {
        self.frames
            .map(DiagnosticHistory::iter)
            .unwrap_or_default()
            .skip(self.offset)
            .take(self.len)
    }

    pub fn first(&self) -> Option<&'a ReplayEvent> {
        self.iter().next()
    }

    pub fn last(&self) -> Option<&'a ReplayEvent> {
        self.iter().next_back()
    }

    pub fn to_owned_slice(&self) -> ReplaySlice {
        ReplaySlice {
            start: self.start,
            end: self.end,
            frames: self.iter().cloned().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReplaySlice {
    pub start: Option<ReplayCursor>,
    pub end: Option<ReplayCursor>,
    pub frames: Vec<ReplayFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SynthesizedReplaySlice {
    pub start: Option<ReplayCursor>,
    pub end: Option<ReplayCursor>,
    pub frames: Vec<ReplayFrame>,
}

impl SynthesizedReplaySlice {
    pub fn new(
        start: Option<ReplayCursor>,
        end: Option<ReplayCursor>,
        frames: Vec<ReplayFrame>,
    ) -> Self {
        Self { start, end, frames }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ReplayFrame> {
        self.frames.iter()
    }

    pub fn first(&self) -> Option<&ReplayFrame> {
        self.frames.first()
    }

    pub fn last(&self) -> Option<&ReplayFrame> {
        self.frames.last()
    }

    pub fn to_owned_slice(&self) -> ReplaySlice {
        ReplaySlice {
            start: self.start,
            end: self.end,
            frames: self.frames.clone(),
        }
    }
}
