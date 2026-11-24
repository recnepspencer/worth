use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::BranchId;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::LineageEventRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageGraphTraversalBasis {
    FullBranchGraphMaterialization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageGraphDigestMode {
    ExactDigestCanonicalOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageNode {
    pub(crate) lineage_id: LineageId,
    pub(crate) entity_id: EntityId,
}

impl LineageNode {
    pub fn lineage_id(&self) -> LineageId {
        self.lineage_id
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageGraphSnapshot {
    pub(crate) branch_id: BranchId,
    pub(crate) nodes: Vec<LineageNode>,
    pub(crate) events: Vec<LineageEventRecord>,
    pub(crate) traversal_basis: LineageGraphTraversalBasis,
    digest_basis: LineageGraphDigestBasis,
    pub(crate) metrics: LineageGraphMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageGraphRequest {
    pub branch_id: BranchId,
    pub traversal_basis: LineageGraphTraversalBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LineageGraphMetrics {
    pub node_count: usize,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageGraphDigestBasis {
    branch_id: BranchId,
    traversal_basis: LineageGraphTraversalBasis,
    canonical_lineage_ids: Vec<LineageId>,
    canonical_event_ids: Vec<u64>,
    digest_mode: LineageGraphDigestMode,
}

impl LineageGraphSnapshot {
    #[cfg(test)]
    pub(crate) fn new(
        branch_id: BranchId,
        nodes: Vec<LineageNode>,
        events: Vec<LineageEventRecord>,
        traversal_basis: LineageGraphTraversalBasis,
        digest_basis: LineageGraphDigestBasis,
        metrics: LineageGraphMetrics,
    ) -> Self {
        Self {
            branch_id,
            nodes,
            events,
            traversal_basis,
            digest_basis,
            metrics,
        }
    }

    pub fn digest_basis(&self) -> &LineageGraphDigestBasis {
        &self.digest_basis
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn nodes(&self) -> &[LineageNode] {
        &self.nodes
    }

    pub fn events(&self) -> &[LineageEventRecord] {
        &self.events
    }

    pub fn traversal_basis(&self) -> LineageGraphTraversalBasis {
        self.traversal_basis
    }

    pub fn metrics(&self) -> &LineageGraphMetrics {
        &self.metrics
    }
}

impl LineageGraphDigestBasis {
    #[cfg(test)]
    pub(crate) fn new(
        branch_id: BranchId,
        traversal_basis: LineageGraphTraversalBasis,
        canonical_lineage_ids: Vec<LineageId>,
        canonical_event_ids: Vec<u64>,
        digest_mode: LineageGraphDigestMode,
    ) -> Self {
        Self {
            branch_id,
            traversal_basis,
            canonical_lineage_ids,
            canonical_event_ids,
            digest_mode,
        }
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn traversal_basis(&self) -> LineageGraphTraversalBasis {
        self.traversal_basis
    }

    pub fn canonical_lineage_ids(&self) -> &[LineageId] {
        &self.canonical_lineage_ids
    }

    pub fn canonical_event_ids(&self) -> &[u64] {
        &self.canonical_event_ids
    }

    pub fn digest_mode(&self) -> LineageGraphDigestMode {
        self.digest_mode
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LineageDivergenceMetrics {
    pub left_event_count: usize,
    pub right_event_count: usize,
    pub left_node_count: usize,
    pub right_node_count: usize,
    pub shared_lineage_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageDivergenceSummary {
    pub(crate) left_branch: BranchId,
    pub(crate) right_branch: BranchId,
    pub(crate) traversal_basis: LineageDivergenceTraversalBasis,
    pub(crate) left_only_event_ids: Vec<u64>,
    pub(crate) right_only_event_ids: Vec<u64>,
    pub(crate) shared_lineage_ids: Vec<LineageId>,
    pub(crate) metrics: LineageDivergenceMetrics,
}

impl LineageDivergenceSummary {
    pub fn left_branch(&self) -> &BranchId {
        &self.left_branch
    }

    pub fn right_branch(&self) -> &BranchId {
        &self.right_branch
    }

    pub fn traversal_basis(&self) -> LineageDivergenceTraversalBasis {
        self.traversal_basis
    }

    pub fn left_only_event_ids(&self) -> &[u64] {
        &self.left_only_event_ids
    }

    pub fn right_only_event_ids(&self) -> &[u64] {
        &self.right_only_event_ids
    }

    pub fn shared_lineage_ids(&self) -> &[LineageId] {
        &self.shared_lineage_ids
    }

    pub fn metrics(&self) -> &LineageDivergenceMetrics {
        &self.metrics
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageDivergenceRequest {
    pub left_branch: BranchId,
    pub right_branch: BranchId,
    pub traversal_basis: LineageDivergenceTraversalBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageDivergenceTraversalBasis {
    FullBranchGraphComparison,
}
