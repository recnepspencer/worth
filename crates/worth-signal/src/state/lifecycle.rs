mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::core_profile::CORE_STORAGE_PROFILE_ID;
use crate::diagnostics::policy::ArtifactRetentionPolicy;
use crate::diagnostics::replay::ReplayCursor;
use crate::runtime_policy::SignalRuntimePolicy;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
/// Stable identifier for a captured runtime-evaluation snapshot.
pub struct SignalSnapshotId(pub u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
/// Stable identifier for a branch-local evaluation timeline.
pub struct SignalBranchId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Public branch handle for snapshot, restore, and replay inspection APIs.
///
/// Branches are runtime-local evaluation timelines. They model branch-local
/// graph state, replay history, and lineage ancestry; they are not durable
/// relational history stores.
pub struct SignalBranchHandle {
    pub id: SignalBranchId,
    pub name: String,
    pub parent_branch_id: Option<SignalBranchId>,
    pub head_snapshot_id: Option<SignalSnapshotId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned metadata for a serialized `SignalSnapshotV1`.
///
/// The snapshot body captures evaluation state, not host-managed source truth.
/// Use this metadata to inspect compatibility, active branch identity, replay
/// cursor position, and runtime policy without restoring the snapshot.
pub struct SignalSnapshotMeta {
    pub schema_version: u32,
    pub snapshot_id: SignalSnapshotId,
    pub branch_id: SignalBranchId,
    pub branch_name: String,
    pub core_storage_profile: String,
    pub replay_head: Option<ReplayCursor>,
    /// Descriptive caller request metadata captured with the snapshot.  It is
    /// not the installed runtime authority used by restore or planning.
    #[serde(rename = "runtime_policy")]
    pub requested_runtime_policy: SignalRuntimePolicy,
    #[serde(default)]
    pub artifact_retention: SnapshotArtifactRetentionPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Explicit snapshot-time retention contract for cold explanation/provenance richness.
///
/// This is a richness policy only. Snapshot restore, identity, and replay truth
/// remain defined by the runtime and dependency contracts, not by retained
/// explanation/provenance payload presence.
pub struct SnapshotArtifactRetentionPolicy {
    pub explanation_retention: ArtifactRetentionPolicy,
    pub provenance_retention: ArtifactRetentionPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Explicit operational intent for snapshot restore.
pub struct SnapshotRestoreIntent {
    pub state: SnapshotStateRestoreMode,
    pub artifacts: SnapshotArtifactRestoreMode,
    pub dependency_state: SnapshotDependencyRestoreMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Whether restore rewinds operational runtime authority to captured state.
pub enum SnapshotStateRestoreMode {
    RewindActiveState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// How cold explanation/provenance richness should be handled during restore.
pub enum SnapshotArtifactRestoreMode {
    RestoreCapturedRetention,
    ApplyActiveRuntimePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Whether dependency state is restored as captured or used only as a recomputation seed.
pub enum SnapshotDependencyRestoreMode {
    RestoreCapturedState,
    SeedRecomputationFromSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reason a restore currently still requires a coarse graph/state replacement boundary.
pub enum SnapshotRestoreCoarseReason {
    EntryStateRewind,
    NodeSetDifference,
    DiagnosticsHistoryRestore,
}

impl SignalSnapshotMeta {
    pub const SCHEMA_VERSION: u32 = 2;

    pub fn new(
        snapshot_id: SignalSnapshotId,
        branch: &SignalBranchHandle,
        replay_head: Option<ReplayCursor>,
        requested_runtime_policy: SignalRuntimePolicy,
        artifact_retention: SnapshotArtifactRetentionPolicy,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            snapshot_id,
            branch_id: branch.id,
            branch_name: branch.name.clone(),
            core_storage_profile: CORE_STORAGE_PROFILE_ID.to_string(),
            replay_head,
            requested_runtime_policy,
            artifact_retention,
        }
    }
}

impl SnapshotArtifactRetentionPolicy {
    pub fn from_runtime_policy(policy: SignalRuntimePolicy) -> Self {
        Self {
            explanation_retention: policy.retention_budget.explanation_retention,
            provenance_retention: policy.retention_budget.provenance_retention,
        }
    }

    pub fn from_retention_budget(budget: crate::diagnostics::policy::RetentionBudget) -> Self {
        Self {
            explanation_retention: budget.explanation_retention,
            provenance_retention: budget.provenance_retention,
        }
    }

    pub fn retains_explanation_facts(self) -> bool {
        matches!(self.explanation_retention, ArtifactRetentionPolicy::Retain)
    }

    pub fn retains_provenance_facts(self) -> bool {
        matches!(self.provenance_retention, ArtifactRetentionPolicy::Retain)
    }
}

impl Default for SnapshotArtifactRetentionPolicy {
    fn default() -> Self {
        Self::from_runtime_policy(SignalRuntimePolicy::default())
    }
}

impl SnapshotRestoreIntent {
    pub fn restore_runtime_truth() -> Self {
        Self {
            state: SnapshotStateRestoreMode::RewindActiveState,
            artifacts: SnapshotArtifactRestoreMode::RestoreCapturedRetention,
            dependency_state: SnapshotDependencyRestoreMode::RestoreCapturedState,
        }
    }

    pub fn restore_runtime_truth_with_active_policy() -> Self {
        Self {
            state: SnapshotStateRestoreMode::RewindActiveState,
            artifacts: SnapshotArtifactRestoreMode::ApplyActiveRuntimePolicy,
            dependency_state: SnapshotDependencyRestoreMode::RestoreCapturedState,
        }
    }
}
