use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::BranchId;
use crate::identity::data::LineageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalResolutionBoundednessBasis {
    BranchScopedLineageSeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalResolutionDigestMode {
    ExactDigestCanonicalOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HistoricalLineageResolutionMetrics {
    pub traversed_event_count: usize,
    pub event_visit_count: usize,
    pub resolved_lineage_count: usize,
    pub lineage_seed_index_probe_count: usize,
    pub reachable_event_index_probe_count: usize,
    pub reachable_commit_node_visits: usize,
    pub reachable_commit_parent_edge_visits: usize,
    pub reachable_commit_catalog_probes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalResolutionRequest {
    pub branch_id: BranchId,
    pub lineage_id: LineageId,
    pub boundedness_basis: HistoricalResolutionBoundednessBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHistoryRequest {
    pub branch_id: BranchId,
    pub entity_id: crate::identity::data::EntityId,
    pub boundedness_basis: HistoricalResolutionBoundednessBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalResolutionTrace {
    pub traversed_event_ids: Vec<u64>,
    pub boundedness_basis: HistoricalResolutionBoundednessBasis,
    digest_basis: HistoricalLineageResolutionDigestBasis,
    pub metrics: HistoricalLineageResolutionMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalLineageResolution {
    pub branch_id: BranchId,
    pub start: LineageId,
    pub resolved: Vec<LineageId>,
    pub boundedness_basis: HistoricalResolutionBoundednessBasis,
    pub traversed_event_ids: Vec<u64>,
    digest_basis: HistoricalLineageResolutionDigestBasis,
    pub trace: HistoricalResolutionTrace,
    pub metrics: HistoricalLineageResolutionMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalLineageResolutionDigestBasis {
    branch_id: BranchId,
    requested_start: LineageId,
    canonical_resolved_lineage_ids: Vec<LineageId>,
    canonical_traversed_event_ids: Vec<u64>,
    boundedness_basis: HistoricalResolutionBoundednessBasis,
    digest_mode: HistoricalResolutionDigestMode,
}

impl HistoricalResolutionTrace {
    pub fn digest_basis(&self) -> &HistoricalLineageResolutionDigestBasis {
        &self.digest_basis
    }
}

impl HistoricalLineageResolution {
    pub fn digest_basis(&self) -> &HistoricalLineageResolutionDigestBasis {
        &self.digest_basis
    }
}

impl HistoricalLineageResolutionDigestBasis {
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn requested_start(&self) -> LineageId {
        self.requested_start
    }

    pub fn canonical_resolved_lineage_ids(&self) -> &[LineageId] {
        &self.canonical_resolved_lineage_ids
    }

    pub fn canonical_traversed_event_ids(&self) -> &[u64] {
        &self.canonical_traversed_event_ids
    }

    pub fn boundedness_basis(&self) -> HistoricalResolutionBoundednessBasis {
        self.boundedness_basis
    }

    pub fn digest_mode(&self) -> HistoricalResolutionDigestMode {
        self.digest_mode
    }
}

/// Operations used by the Relational owners.
pub trait HistoricalResolutionTraceOwnerPort: Sized {
    fn new(
        traversed_event_ids: Vec<u64>,
        boundedness_basis: HistoricalResolutionBoundednessBasis,
        digest_basis: HistoricalLineageResolutionDigestBasis,
        metrics: HistoricalLineageResolutionMetrics,
    ) -> Self;
}

impl HistoricalResolutionTraceOwnerPort for HistoricalResolutionTrace {
    fn new(
        traversed_event_ids: Vec<u64>,
        boundedness_basis: HistoricalResolutionBoundednessBasis,
        digest_basis: HistoricalLineageResolutionDigestBasis,
        metrics: HistoricalLineageResolutionMetrics,
    ) -> Self {
        Self {
            traversed_event_ids,
            boundedness_basis,
            digest_basis,
            metrics,
        }
    }
}

/// Operations used by the Relational owners.
pub trait HistoricalLineageResolutionOwnerPort: Sized {
    fn new(
        branch_id: BranchId,
        start: LineageId,
        resolved: Vec<LineageId>,
        boundedness_basis: HistoricalResolutionBoundednessBasis,
        traversed_event_ids: Vec<u64>,
        digest_basis: HistoricalLineageResolutionDigestBasis,
        trace: HistoricalResolutionTrace,
        metrics: HistoricalLineageResolutionMetrics,
    ) -> Self;
}

impl HistoricalLineageResolutionOwnerPort for HistoricalLineageResolution {
    fn new(
        branch_id: BranchId,
        start: LineageId,
        resolved: Vec<LineageId>,
        boundedness_basis: HistoricalResolutionBoundednessBasis,
        traversed_event_ids: Vec<u64>,
        digest_basis: HistoricalLineageResolutionDigestBasis,
        trace: HistoricalResolutionTrace,
        metrics: HistoricalLineageResolutionMetrics,
    ) -> Self {
        Self {
            branch_id,
            start,
            resolved,
            boundedness_basis,
            traversed_event_ids,
            digest_basis,
            trace,
            metrics,
        }
    }
}

/// Operations used by the Relational owners.
pub trait HistoricalLineageResolutionDigestBasisOwnerPort: Sized {
    fn new(
        branch_id: BranchId,
        requested_start: LineageId,
        canonical_resolved_lineage_ids: Vec<LineageId>,
        canonical_traversed_event_ids: Vec<u64>,
        boundedness_basis: HistoricalResolutionBoundednessBasis,
        digest_mode: HistoricalResolutionDigestMode,
    ) -> Self;
}

impl HistoricalLineageResolutionDigestBasisOwnerPort for HistoricalLineageResolutionDigestBasis {
    fn new(
        branch_id: BranchId,
        requested_start: LineageId,
        canonical_resolved_lineage_ids: Vec<LineageId>,
        canonical_traversed_event_ids: Vec<u64>,
        boundedness_basis: HistoricalResolutionBoundednessBasis,
        digest_mode: HistoricalResolutionDigestMode,
    ) -> Self {
        Self {
            branch_id,
            requested_start,
            canonical_resolved_lineage_ids,
            canonical_traversed_event_ids,
            boundedness_basis,
            digest_mode,
        }
    }
}
