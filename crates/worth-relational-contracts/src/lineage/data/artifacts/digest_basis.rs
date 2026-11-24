use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::{BranchId, CommitId};
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{LineageEventKind, LineageEventRecord};

use super::{
    FinalizedLineageEventBatch, LineageDecisionKind, LineageDecisionLog, LineageDecisionRecord,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageDigestBasis {
    branch_id: BranchId,
    canonical_event_ids: Vec<u64>,
    lineage_event_count: usize,
    lineage_decision_count: usize,
    event_batch: LineageEventBatchDigestBasis,
    decision_log: LineageDecisionLogDigestBasis,
}

impl LineageDigestBasis {
    pub(super) fn new(
        branch_id: BranchId,
        canonical_event_ids: Vec<u64>,
        lineage_event_count: usize,
        lineage_decision_count: usize,
        event_batch: LineageEventBatchDigestBasis,
        decision_log: LineageDecisionLogDigestBasis,
    ) -> Self {
        Self {
            branch_id,
            canonical_event_ids,
            lineage_event_count,
            lineage_decision_count,
            event_batch,
            decision_log,
        }
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn canonical_event_ids(&self) -> &[u64] {
        &self.canonical_event_ids
    }

    pub fn lineage_event_count(&self) -> usize {
        self.lineage_event_count
    }

    pub fn lineage_decision_count(&self) -> usize {
        self.lineage_decision_count
    }

    pub fn event_batch(&self) -> &LineageEventBatchDigestBasis {
        &self.event_batch
    }

    pub fn decision_log(&self) -> &LineageDecisionLogDigestBasis {
        &self.decision_log
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageEventBatchDigestBasis {
    branch_id: BranchId,
    canonical_event_ids: Vec<u64>,
    canonical_commit_ids: Vec<CommitId>,
    canonical_event_kinds: Vec<LineageEventKind>,
    canonical_source_orderings: Vec<Vec<LineageId>>,
    canonical_target_orderings: Vec<Vec<LineageId>>,
    /// Every explicit `(event, entity)` binding; empty for every commit
    /// without a selection restore, so those digests never change.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    canonical_entity_bindings: Vec<(u64, EntityId)>,
}

impl LineageEventBatchDigestBasis {
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn canonical_event_ids(&self) -> &[u64] {
        &self.canonical_event_ids
    }

    pub fn canonical_commit_ids(&self) -> &[CommitId] {
        &self.canonical_commit_ids
    }

    pub fn canonical_event_kinds(&self) -> &[LineageEventKind] {
        &self.canonical_event_kinds
    }

    pub fn canonical_source_orderings(&self) -> &[Vec<LineageId>] {
        &self.canonical_source_orderings
    }

    pub fn canonical_target_orderings(&self) -> &[Vec<LineageId>] {
        &self.canonical_target_orderings
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageDecisionLogDigestBasis {
    branch_id: BranchId,
    canonical_decision_kinds: Vec<LineageDecisionKind>,
    canonical_event_ids: Vec<Option<u64>>,
    canonical_source_orderings: Vec<Vec<LineageId>>,
    canonical_target_orderings: Vec<Vec<LineageId>>,
}

impl LineageDecisionLogDigestBasis {
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn canonical_decision_kinds(&self) -> &[LineageDecisionKind] {
        &self.canonical_decision_kinds
    }

    pub fn canonical_event_ids(&self) -> &[Option<u64>] {
        &self.canonical_event_ids
    }

    pub fn canonical_source_orderings(&self) -> &[Vec<LineageId>] {
        &self.canonical_source_orderings
    }

    pub fn canonical_target_orderings(&self) -> &[Vec<LineageId>] {
        &self.canonical_target_orderings
    }
}

pub(super) fn event_batch_digest_basis(
    branch_id: &BranchId,
    event_batch: &FinalizedLineageEventBatch,
) -> LineageEventBatchDigestBasis {
    event_batch_digest_basis_from_parts(branch_id, event_batch.event_ids(), event_batch.events())
}

pub(super) fn event_batch_digest_basis_from_parts(
    branch_id: &BranchId,
    event_ids: &[u64],
    events: &[LineageEventRecord],
) -> LineageEventBatchDigestBasis {
    LineageEventBatchDigestBasis {
        branch_id: branch_id.clone(),
        canonical_event_ids: event_ids.to_vec(),
        canonical_commit_ids: events.iter().map(|event| event.commit.commit_id).collect(),
        canonical_event_kinds: events.iter().map(|event| event.kind).collect(),
        canonical_source_orderings: events.iter().map(|event| event.sources.clone()).collect(),
        canonical_target_orderings: events.iter().map(|event| event.targets.clone()).collect(),
        canonical_entity_bindings: events
            .iter()
            .filter_map(|event| event.entity_id.map(|entity| (event.event_id, entity)))
            .collect(),
    }
}

pub(super) fn decision_log_digest_basis(
    branch_id: &BranchId,
    decision_log: &LineageDecisionLog,
) -> LineageDecisionLogDigestBasis {
    decision_log_digest_basis_from_parts(branch_id, decision_log.decisions())
}

pub(super) fn decision_log_digest_basis_from_parts(
    branch_id: &BranchId,
    decisions: &[LineageDecisionRecord],
) -> LineageDecisionLogDigestBasis {
    LineageDecisionLogDigestBasis {
        branch_id: branch_id.clone(),
        canonical_decision_kinds: decisions
            .iter()
            .map(|decision| decision.kind().clone())
            .collect(),
        canonical_event_ids: decisions
            .iter()
            .map(|decision| decision.event_id())
            .collect(),
        canonical_source_orderings: decisions
            .iter()
            .map(|decision| decision.sources().to_vec())
            .collect(),
        canonical_target_orderings: decisions
            .iter()
            .map(|decision| decision.targets().to_vec())
            .collect(),
    }
}
