use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::BranchId;
use crate::identity::data::{LineageId, RelationId};

use super::digest_basis::{decision_log_digest_basis, event_batch_digest_basis};
use super::{
    FinalizedLineageEventBatch, LineageArtifactCounters, LineageDecisionLog, LineageDigestBasis,
    PublishedLineageArtifact,
};

pub struct PreparedLineageFinalization {
    artifact: LineageFinalizationArtifact,
    new_nodes: Vec<crate::lineage::data::LineageNode>,
    entity_bindings: Vec<(crate::identity::data::EntityId, LineageId)>,
    new_relation_lineages: BTreeMap<RelationId, LineageId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageFinalizationArtifact {
    branch_id: BranchId,
    event_batch: FinalizedLineageEventBatch,
    decision_log: LineageDecisionLog,
    digest_basis: LineageDigestBasis,
    counters: LineageArtifactCounters,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    relation_lineages: Vec<(RelationId, LineageId)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    group_lineage: Vec<u8>,
}

impl LineageFinalizationArtifact {
    pub fn new(
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
            relation_lineages: Vec::new(),
            group_lineage: Vec::new(),
        }
    }

    pub fn with_relation_lineages(
        mut self,
        relation_lineages: Vec<(RelationId, LineageId)>,
    ) -> Self {
        self.relation_lineages = relation_lineages;
        self
    }

    /// Attaches this commit's canonical group lineage frame.
    pub fn with_group_lineage(mut self, group_lineage: Vec<u8>) -> Self {
        self.group_lineage = group_lineage;
        self
    }

    pub fn event_batch(&self) -> &FinalizedLineageEventBatch {
        &self.event_batch
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn decision_log(&self) -> &LineageDecisionLog {
        &self.decision_log
    }

    pub fn digest_basis(&self) -> &LineageDigestBasis {
        &self.digest_basis
    }

    pub fn counters(&self) -> &LineageArtifactCounters {
        &self.counters
    }

    pub fn publish(&self) -> PublishedLineageArtifact {
        PublishedLineageArtifact::new(
            self.branch_id.clone(),
            self.event_batch().event_ids().to_vec(),
            self.event_batch().events().to_vec(),
            self.decision_log().decisions().to_vec(),
            self.digest_basis().clone(),
            *self.counters(),
        )
        .with_relation_lineages(self.relation_lineages.clone())
        .with_group_lineage(self.group_lineage.clone())
    }
}

impl PreparedLineageFinalization {
    pub fn new(
        artifact: LineageFinalizationArtifact,
        new_nodes: Vec<crate::lineage::data::LineageNode>,
    ) -> Self {
        Self {
            artifact,
            new_nodes,
            entity_bindings: Vec::new(),
            new_relation_lineages: BTreeMap::new(),
        }
    }

    pub fn with_relation_lineages(
        artifact: LineageFinalizationArtifact,
        new_nodes: Vec<crate::lineage::data::LineageNode>,
        new_relation_lineages: BTreeMap<RelationId, LineageId>,
    ) -> Self {
        Self {
            artifact,
            new_nodes,
            entity_bindings: Vec::new(),
            new_relation_lineages,
        }
    }

    pub fn with_entity_bindings(
        mut self,
        bindings: Vec<(crate::identity::data::EntityId, LineageId)>,
    ) -> Self {
        self.entity_bindings = bindings;
        self
    }

    /// Attaches the commit's canonical group lineage frame.
    pub fn attach_group_lineage(&mut self, group_lineage: Vec<u8>) {
        self.artifact.group_lineage = group_lineage;
    }

    pub fn take_entity_bindings(&mut self) -> Vec<(crate::identity::data::EntityId, LineageId)> {
        std::mem::take(&mut self.entity_bindings)
    }

    pub fn artifact(&self) -> &LineageFinalizationArtifact {
        &self.artifact
    }

    pub(crate) fn relation_lineages(&self) -> &BTreeMap<RelationId, LineageId> {
        &self.new_relation_lineages
    }

    pub fn new_nodes(&self) -> &[crate::lineage::data::LineageNode] {
        &self.new_nodes
    }

    pub fn into_parts(
        self,
    ) -> (
        LineageFinalizationArtifact,
        Vec<crate::lineage::data::LineageNode>,
        Vec<(RelationId, LineageId)>,
    ) {
        debug_assert!(self.entity_bindings.is_empty());
        (
            self.artifact,
            self.new_nodes,
            self.new_relation_lineages.into_iter().collect(),
        )
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait PreparedLineageFinalizationFixturePort: Sized {
    fn new_nodes_for_test(&self) -> &[crate::lineage::data::LineageNode];
    fn relation_lineages_for_test(&self) -> &BTreeMap<RelationId, LineageId>;
}

impl PreparedLineageFinalizationFixturePort for PreparedLineageFinalization {
    fn new_nodes_for_test(&self) -> &[crate::lineage::data::LineageNode] {
        self.new_nodes()
    }

    fn relation_lineages_for_test(&self) -> &BTreeMap<RelationId, LineageId> {
        self.relation_lineages()
    }
}
