use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::handle::NodeId;
use crate::data::output::CanonicalChangedRegions;
use crate::data::reuse::{ReuseBoundaryContext, ReuseCertificationRecord};

use super::evidence::CausalityMetadata;
use super::tiers::RuntimeArtifactState;
use super::writes::ColdArtifactRecord;

/// Cold execution/segment metadata used by diagnostics and replay-facing
/// summaries, but not required for hot mutation semantics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExecutionTraceStamp {
    #[serde(default)]
    pub execution_record_id: Option<u64>,
    #[serde(default)]
    pub semantic_segment_id: Option<u64>,
}

/// Cold retained artifact richness kept off the operational hot path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RetainedDiagnosticArtifact {
    /// Generic changed-region metadata retained for diagnostics and explain
    /// reconstruction.
    #[serde(default)]
    pub changed_regions: CanonicalChangedRegions,
    /// Optional structured labels for diagnostics.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Family namespace for keyed computations, when relevant.
    #[serde(default)]
    pub keyed_family: Option<String>,
    /// Key inside the computation family, when relevant.
    #[serde(default)]
    pub keyed_key: Option<String>,
    /// Full cold-path proof for why reuse was legal, when retained.
    #[serde(default)]
    pub reuse_certification: Option<ReuseCertificationRecord>,
    /// Rich cold reuse boundary detail retained for explanation/forensics.
    #[serde(default)]
    pub reuse_boundary_context: Option<ReuseBoundaryContext>,
}

/// Cold historical artifact record assembled for explanation, lineage
/// expansion, and retained reporting surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoricalArtifactRecord {
    pub node: NodeId,
    pub runtime: RuntimeArtifactState,
    #[serde(default)]
    pub retained: Option<ColdArtifactRecord>,
    #[serde(default)]
    pub causality: Option<CausalityMetadata>,
}
