use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::identity::data::{LineageId, RelationId};
use crate::lineage::data::LineageEventRecord;

use super::digest_basis::{
    decision_log_digest_basis_from_parts, event_batch_digest_basis_from_parts,
};
use super::{
    LineageArtifactCounters, LineageDecisionLogDigestBasis, LineageDecisionRecord,
    LineageDigestBasis, LineageEventBatchDigestBasis,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PublishedLineageArtifact {
    branch_id: BranchId,
    lineage_event_ids: Vec<u64>,
    lineage_events: Vec<LineageEventRecord>,
    lineage_decision_log: Vec<LineageDecisionRecord>,
    digest_basis: LineageDigestBasis,
    counters: LineageArtifactCounters,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    relation_lineages: Vec<(RelationId, LineageId)>,
}

impl PublishedLineageArtifact {
    pub(super) fn new(
        branch_id: BranchId,
        lineage_event_ids: Vec<u64>,
        lineage_events: Vec<LineageEventRecord>,
        lineage_decision_log: Vec<LineageDecisionRecord>,
        digest_basis: LineageDigestBasis,
        counters: LineageArtifactCounters,
    ) -> Self {
        Self {
            branch_id,
            lineage_event_ids,
            lineage_events,
            lineage_decision_log,
            digest_basis,
            counters,
            relation_lineages: Vec::new(),
        }
    }

    pub(super) fn with_relation_lineages(
        mut self,
        relation_lineages: Vec<(RelationId, LineageId)>,
    ) -> Self {
        self.relation_lineages = relation_lineages;
        self
    }

    pub(crate) fn relation_lineages(&self) -> &[(RelationId, LineageId)] {
        &self.relation_lineages
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub(crate) fn lineage_event_ids(&self) -> &[u64] {
        &self.lineage_event_ids
    }

    pub(crate) fn lineage_events(&self) -> &[LineageEventRecord] {
        &self.lineage_events
    }

    pub(crate) fn lineage_decision_log(&self) -> &[LineageDecisionRecord] {
        &self.lineage_decision_log
    }

    pub(crate) fn decisions_for_event_id(
        &self,
        event_id: u64,
    ) -> impl Iterator<Item = &LineageDecisionRecord> {
        self.lineage_decision_log
            .iter()
            .filter(move |decision| decision.event_id() == Some(event_id))
    }

    pub(crate) fn digest_basis(&self) -> &LineageDigestBasis {
        &self.digest_basis
    }

    pub(crate) fn event_batch_digest_basis(&self) -> &LineageEventBatchDigestBasis {
        self.digest_basis.event_batch()
    }

    pub(crate) fn decision_log_digest_basis(&self) -> &LineageDecisionLogDigestBasis {
        self.digest_basis.decision_log()
    }

    pub(crate) fn observed_event_batch_digest_basis(&self) -> LineageEventBatchDigestBasis {
        event_batch_digest_basis_from_parts(
            &self.branch_id,
            &self.lineage_event_ids,
            &self.lineage_events,
        )
    }

    pub(crate) fn observed_decision_log_digest_basis(&self) -> LineageDecisionLogDigestBasis {
        decision_log_digest_basis_from_parts(&self.branch_id, &self.lineage_decision_log)
    }

    pub(crate) fn counters(&self) -> LineageArtifactCounters {
        self.counters
    }

    pub(crate) fn has_authority_content(&self) -> bool {
        !self.lineage_events.is_empty() || !self.lineage_decision_log.is_empty()
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.branch_id
            .0
            .capacity()
            .try_into()
            .unwrap_or(u64::MAX)
            .saturating_add(vector_capacity_bytes(&self.lineage_event_ids))
            .saturating_add(vector_capacity_bytes(&self.lineage_events))
            .saturating_add(vector_capacity_bytes(&self.lineage_decision_log))
    }

    #[cfg(test)]
    pub(crate) fn lineage_events_mut(&mut self) -> &mut Vec<LineageEventRecord> {
        &mut self.lineage_events
    }

    #[cfg(test)]
    pub(crate) fn lineage_decision_log_mut(&mut self) -> &mut Vec<LineageDecisionRecord> {
        &mut self.lineage_decision_log
    }
}

fn vector_capacity_bytes<T>(values: &Vec<T>) -> u64 {
    (values.capacity() as u64).saturating_mul(std::mem::size_of::<T>() as u64)
}
