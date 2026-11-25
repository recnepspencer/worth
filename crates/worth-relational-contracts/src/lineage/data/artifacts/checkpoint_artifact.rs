use serde::{Deserialize, Serialize};

use crate::identity::data::{LineageId, RelationId};
use crate::lineage::data::LineageNode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageCheckpointDigestBasis {
    pub published_lineage_commit_count: usize,
    pub canonical_published_event_ids: Vec<u64>,
    pub published_lineage_event_count: usize,
    pub published_lineage_decision_count: usize,
}

impl LineageCheckpointDigestBasis {
    pub fn new(
        published_lineage_commit_count: usize,
        canonical_published_event_ids: Vec<u64>,
        published_lineage_event_count: usize,
        published_lineage_decision_count: usize,
    ) -> Self {
        Self {
            published_lineage_commit_count,
            canonical_published_event_ids,
            published_lineage_event_count,
            published_lineage_decision_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageCheckpointCounters {
    pub node_count: usize,
}

impl LineageCheckpointCounters {
    fn new(node_count: usize) -> Self {
        Self { node_count }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageCheckpointArtifact {
    digest_basis: LineageCheckpointDigestBasis,
    counters: LineageCheckpointCounters,
    nodes: Vec<LineageNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    relation_lineages: Vec<(RelationId, LineageId)>,
}

impl LineageCheckpointArtifact {
    pub fn new(digest_basis: LineageCheckpointDigestBasis, nodes: Vec<LineageNode>) -> Self {
        let counters = LineageCheckpointCounters::new(nodes.len());
        Self {
            digest_basis,
            counters,
            nodes,
            relation_lineages: Vec::new(),
        }
    }

    pub fn with_relation_lineages(
        mut self,
        relation_lineages: Vec<(RelationId, LineageId)>,
    ) -> Self {
        self.relation_lineages = relation_lineages;
        self
    }

    pub fn relation_lineages(&self) -> &[(RelationId, LineageId)] {
        &self.relation_lineages
    }

    pub fn digest_basis(&self) -> &LineageCheckpointDigestBasis {
        &self.digest_basis
    }

    pub fn counters(&self) -> LineageCheckpointCounters {
        self.counters
    }

    pub fn nodes(&self) -> &[LineageNode] {
        &self.nodes
    }
}

/// Operations used by the Relational owners.
pub trait LineageCheckpointArtifactOwnerPort: Sized {
    fn checkpoint_nodes_mut_for_test(&mut self) -> &mut Vec<LineageNode>;
}

impl LineageCheckpointArtifactOwnerPort for LineageCheckpointArtifact {
    fn checkpoint_nodes_mut_for_test(&mut self) -> &mut Vec<LineageNode> {
        &mut self.nodes
    }
}
