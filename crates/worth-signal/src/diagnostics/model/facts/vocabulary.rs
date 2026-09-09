mod retained_charge;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::diagnostics::policy::DiagnosticsAvailability;
use crate::logic::explain::NodeExplanation;
use crate::logic::explain::{CausalLink, RewiringSummary};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationFact {
    pub node: NodeId,
    pub explanation: NodeExplanation,
    #[serde(default)]
    pub compact_projection: bool,
    pub materialization_mode: DiagnosticsAvailability,
    pub execution_record_id: Option<u64>,
    pub semantic_segment_id: Option<u64>,
    pub state: String,
    pub upstream_count: u32,
    pub propagation_suppressed: bool,
    pub changed_region_count: u32,
    pub output_change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceFact {
    pub node: NodeId,
    pub materialization_mode: DiagnosticsAvailability,
    pub execution_record_id: Option<u64>,
    pub semantic_segment_id: Option<u64>,
    pub vertices: Vec<ProvenanceVertex>,
    pub edges: Vec<ProvenanceEdge>,
    pub causal_links: Vec<CausalLink>,
    pub rewiring: Option<RewiringSummary>,
    pub propagation_suppressed: bool,
    pub causality_kind: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProvenanceVertexRole {
    Target,
    Upstream,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProvenanceEdgeKind {
    Changed,
    SkippedByComparator,
    ConditionDeferred,
    Clean,
    MissingSnapshot,
    DependencyRemoved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceVertex {
    pub node: NodeId,
    pub role: ProvenanceVertexRole,
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceEdge {
    pub kind: ProvenanceEdgeKind,
    pub source: NodeId,
    pub aspect: Aspect,
    pub subscription: Option<PartitionSubscription>,
    pub cached_version: Option<u64>,
    pub current_version: Option<u64>,
    pub comparator: Option<String>,
    pub reason: Option<String>,
}

pub type ExplanationFactTable = BTreeMap<NodeId, ExplanationFact>;
pub type ProvenanceFactTable = BTreeMap<NodeId, ProvenanceFact>;
