use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::identity::data::{LineageId, RelationId};

use super::digest_basis::{decision_log_digest_basis, event_batch_digest_basis};
use super::{
    FinalizedLineageEventBatch, LineageArtifactCounters, LineageDecisionLog, LineageDigestBasis,
    PublishedLineageArtifact,
};

pub(crate) struct PreparedLineageFinalization {
    artifact: LineageFinalizationArtifact,
    new_nodes: Vec<crate::lineage::data::LineageNode>,
    new_relation_lineages: BTreeMap<RelationId, LineageId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LineageFinalizationArtifact {
    branch_id: BranchId,
    event_batch: FinalizedLineageEventBatch,
    decision_log: LineageDecisionLog,
    digest_basis: LineageDigestBasis,
    counters: LineageArtifactCounters,
}

impl LineageFinalizationArtifact {
    pub(crate) fn new(
        branch_id: BranchId,
        event_batch: FinalizedLineageEventBatch,
        decision_log: LineageDecisionLog,
    ) -> Self {
        let event_batch_basis = event_batch_digest_basis(&branch_id, &event_batch);
        let decision_log_basis = decision_log_digest_basis(&branch_id, &decision_log);
        let digest_basis = LineageDigestBasis::new(
            branch_id.clone(),
            event_batch.event_ids().to_vec(),
            event_batch.events().len(),
            decision_log.decisions().len(),
            event_batch_basis,
            decision_log_basis,
        );
        let counters =
            LineageArtifactCounters::new(*event_batch.counters(), decision_log.decisions().len());
        Self {
            branch_id,
            event_batch,
            decision_log,
            digest_basis,
            counters,
        }
    }

    pub(crate) fn event_batch(&self) -> &FinalizedLineageEventBatch {
        &self.event_batch
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub(crate) fn decision_log(&self) -> &LineageDecisionLog {
        &self.decision_log
    }

    pub(crate) fn digest_basis(&self) -> &LineageDigestBasis {
        &self.digest_basis
    }

    pub(crate) fn counters(&self) -> &LineageArtifactCounters {
        &self.counters
    }

    pub(crate) fn publish(&self) -> PublishedLineageArtifact {
        PublishedLineageArtifact::new(
            self.branch_id.clone(),
            self.event_batch().event_ids().to_vec(),
            self.event_batch().events().to_vec(),
            self.decision_log().decisions().to_vec(),
            self.digest_basis().clone(),
            *self.counters(),
        )
    }
}

impl PreparedLineageFinalization {
    pub(crate) fn new(
        artifact: LineageFinalizationArtifact,
        new_nodes: Vec<crate::lineage::data::LineageNode>,
    ) -> Self {
        Self {
            artifact,
            new_nodes,
            new_relation_lineages: BTreeMap::new(),
        }
    }

    pub(crate) fn with_relation_lineages(
        artifact: LineageFinalizationArtifact,
        new_nodes: Vec<crate::lineage::data::LineageNode>,
        new_relation_lineages: BTreeMap<RelationId, LineageId>,
    ) -> Self {
        Self {
            artifact,
            new_nodes,
            new_relation_lineages,
        }
    }

    pub(crate) fn artifact(&self) -> &LineageFinalizationArtifact {
        &self.artifact
    }

    #[allow(dead_code)]
    pub(crate) fn relation_lineages(&self) -> &BTreeMap<RelationId, LineageId> {
        &self.new_relation_lineages
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        LineageFinalizationArtifact,
        Vec<crate::lineage::data::LineageNode>,
    ) {
        (self.artifact, self.new_nodes)
    }
}
