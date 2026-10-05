use serde::{Deserialize, Serialize};

use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::durability::data::{DurabilityMode, DurableStoreLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationalRuntimeProfile {
    CertificationCore,
    GeometryKernel,
    ChipSimulation,
    AiWorkflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeExecutionLane {
    OperationalThin,
    RichInteractive,
    AuditReplayHeavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticsBoundary {
    MinimalHotTruth,
    RichCertification,
    DurableWorkflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfileBoundaryPolicy {
    pub execution_lane: RuntimeExecutionLane,
    pub diagnostics_boundary: DiagnosticsBoundary,
    pub prefers_checkpoint_compaction: bool,
    pub allows_compiled_lane: bool,
    pub keeps_replay_hot_path_thin: bool,
}

impl RelationalRuntimeProfile {
    pub fn boundary_policy(self) -> RuntimeProfileBoundaryPolicy {
        match self {
            Self::CertificationCore => RuntimeProfileBoundaryPolicy {
                execution_lane: RuntimeExecutionLane::RichInteractive,
                diagnostics_boundary: DiagnosticsBoundary::RichCertification,
                prefers_checkpoint_compaction: false,
                allows_compiled_lane: false,
                keeps_replay_hot_path_thin: true,
            },
            Self::GeometryKernel => RuntimeProfileBoundaryPolicy {
                execution_lane: RuntimeExecutionLane::RichInteractive,
                diagnostics_boundary: DiagnosticsBoundary::RichCertification,
                prefers_checkpoint_compaction: true,
                allows_compiled_lane: false,
                keeps_replay_hot_path_thin: true,
            },
            Self::ChipSimulation => RuntimeProfileBoundaryPolicy {
                execution_lane: RuntimeExecutionLane::OperationalThin,
                diagnostics_boundary: DiagnosticsBoundary::MinimalHotTruth,
                prefers_checkpoint_compaction: true,
                allows_compiled_lane: true,
                keeps_replay_hot_path_thin: true,
            },
            Self::AiWorkflow => RuntimeProfileBoundaryPolicy {
                execution_lane: RuntimeExecutionLane::AuditReplayHeavy,
                diagnostics_boundary: DiagnosticsBoundary::DurableWorkflow,
                prefers_checkpoint_compaction: true,
                allows_compiled_lane: false,
                keeps_replay_hot_path_thin: false,
            },
        }
    }

    pub fn default_diagnostics_profile(self) -> RelationalDiagnosticsProfile {
        match self {
            Self::CertificationCore => RelationalDiagnosticsProfile {
                detailed_traces_enabled: true,
                collect_all_invariant_failures: false,
                max_entries_per_artifact: 512,
                ..RelationalDiagnosticsProfile::default()
            },
            Self::GeometryKernel => RelationalDiagnosticsProfile::geometry_rich_certification(),
            Self::ChipSimulation => RelationalDiagnosticsProfile::chip_operational_hot_path(),
            Self::AiWorkflow => RelationalDiagnosticsProfile {
                detailed_traces_enabled: false,
                collect_all_invariant_failures: false,
                max_entries_per_artifact: 256,
                ..RelationalDiagnosticsProfile::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotReleasePolicy {
    ExplicitRelease,
    ReleaseOnRetentionPass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionBackend {
    PinTrackedRetention,
    EpochChunkRetention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub backend: RetentionBackend,
    pub reclaim_batch_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MvccConfig {
    pub track_visibility_metadata: bool,
    pub snapshot_release_policy: SnapshotReleasePolicy,
    pub auto_reclaim_deleted_records: bool,
    pub reclaim_batch_size: usize,
    pub retention_backend: RetentionBackend,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibilityCachePolicy {
    pub enabled: bool,
    pub protect_branch_heads: bool,
    pub protect_replay_retained: bool,
    pub protect_active_snapshots: bool,
    pub recent_version_window: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjacencyBackend {
    InlineSmallDegreeAdjacency,
    CompressedFanoutAdjacency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdjacencyPolicy {
    pub backend: AdjacencyBackend,
    pub small_degree_inline_capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompiledLanePolicy {
    Disabled,
    DerivedCompiledLane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurableLogRetentionMode {
    RetainAllInMemory,
    CompactAfterCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableLogPolicy {
    pub retention_mode: DurableLogRetentionMode,
    pub max_in_memory_envelopes: usize,
    pub compact_after_checkpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointPolicy {
    pub compact_after_checkpoint: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurabilityPolicy {
    pub mode: DurabilityMode,
    pub log: DurableLogPolicy,
    pub checkpoints: CheckpointPolicy,
    pub store_layout: Option<DurableStoreLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageLayoutConfig {
    pub entity_chunk_size: usize,
    pub relation_chunk_size: usize,
    pub scan_packet_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CrossContextPolicy {
    AllowExplicit,
    SchemaControlled,
    Forbid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CascadeDeletePolicy {
    RetainDanglingForAudit,
    CascadeDeleteRelations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MutationConfig {
    pub(crate) cascade_delete_policy: CascadeDeletePolicy,
    pub(crate) adjacency_policy: AdjacencyPolicy,
    pub(crate) cross_context_policy: CrossContextPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationConfig {
    pub coherent_publication_required: bool,
    pub max_patch_records_per_commit: usize,
    pub max_published_snapshot_handles: usize,
    pub max_active_snapshot_handles: usize,
    pub max_transaction_overlay_bytes: u64,
    pub max_transaction_footprint_loci: usize,
    pub max_transaction_savepoints: usize,
    pub max_prepared_candidates: usize,
    pub candidate_max_lifetime_millis: u64,
    pub max_prepared_root_bytes: u64,
}
