use serde::{Deserialize, Serialize};

mod deserialization;
mod retained_charge;

use crate::data::core_profile::StableHashValue;
use crate::data::output::{
    ArtifactContinuityToken, MemoizedResultOrigin, OutputChange, OutputIdentity,
};
use crate::data::reuse::{ReuseBasis, ReuseBoundaryAuthority, ReuseOrigin};
use crate::diagnostics::lineage::LineageArtifactId;

use super::authority::{
    ArtifactMergeAuthority, ArtifactTransitionKey, CompactChangedScopeProof,
    ContinuityAuthorityToken, ReuseOperationalBasis,
};

/// Compact hot operational artifact truth retained directly on the node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuntimeArtifactHot {
    /// Opaque deterministic hash for the evaluated output.
    pub output_hash: StableHashValue,
    /// Runtime-normalized output change classification.
    #[serde(default)]
    pub output_change: OutputChange,
    /// Whether this node executed `compute` during the last evaluation.
    #[serde(default)]
    pub recomputed: bool,
    /// Number of dependencies observed during the last clean evaluation.
    #[serde(default)]
    pub dependency_count: u32,
    /// Number of upstream inputs that differed from the cached snapshot.
    #[serde(default)]
    pub meaningful_input_changes: u32,
    /// Number of distinct partitions reported as changed.
    #[serde(default)]
    pub changed_partition_count: u32,
    /// Whether downstream invalidation was suppressed after evaluation.
    #[serde(default)]
    pub propagation_suppressed: bool,
    /// Narrowed locality proof for partition-aware runtime behavior.
    #[serde(default)]
    pub changed_scopes: CompactChangedScopeProof,
}

/// Warm operational artifact metadata retained with the node, but not intended
/// for the tightest hot loops.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuntimeArtifactWarm {
    /// Optional stable identity for the evaluated output artifact.
    #[serde(default)]
    pub output_identity: Option<OutputIdentity>,
    /// Optional host-defined continuity token for lineage preservation.
    #[serde(default)]
    pub continuity_token: ContinuityAuthorityToken,
    /// How the last result was produced.
    #[serde(default)]
    pub memoized_origin: MemoizedResultOrigin,
    /// Compact runtime truth for how the current artifact became current.
    #[serde(default)]
    pub reuse_basis: ReuseOperationalBasis,
    /// Realized runtime origin for how the current artifact became current.
    #[serde(default)]
    pub reuse_origin: ReuseOrigin,
    /// Compact hot authority for certifying later reuse of this artifact.
    #[serde(default)]
    pub reuse_boundary_authority: Option<ReuseBoundaryAuthority>,
    /// Current signal-lineage artifact id for this node's evaluated artifact.
    #[serde(default)]
    pub lineage_artifact_id: ArtifactTransitionKey,
    /// Typed authority/adoptability truth used by branch merge semantics.
    #[serde(default)]
    pub merge_authority: ArtifactMergeAuthority,
}

/// Runtime artifact state split into hot operational truth and warm companion
/// metadata while preserving a flat serialized schema boundary.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct RuntimeArtifactState {
    #[serde(flatten)]
    hot: RuntimeArtifactHot,
    #[serde(flatten)]
    warm: RuntimeArtifactWarm,
}

/// Compact planner/finalize-facing runtime artifact image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifactFinalizeImage {
    hot: RuntimeArtifactHot,
    reuse_origin: ReuseOrigin,
    reuse_boundary_authority: Option<ReuseBoundaryAuthority>,
    lineage_artifact_id: ArtifactTransitionKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifactReuseBoundarySnapshot {
    pub output_identity: Option<OutputIdentity>,
    pub continuity_token: Option<ArtifactContinuityToken>,
    pub reuse_boundary_authority: Option<ReuseBoundaryAuthority>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifactOperationalSummary {
    pub memoized_origin: MemoizedResultOrigin,
    pub reuse_basis: ReuseBasis,
    pub reuse_origin: ReuseOrigin,
}

impl RuntimeArtifactState {
    pub fn new(hot: RuntimeArtifactHot, warm: RuntimeArtifactWarm) -> Self {
        Self { hot, warm }
    }

    pub fn hot(&self) -> &RuntimeArtifactHot {
        &self.hot
    }

    pub fn hot_mut(&mut self) -> &mut RuntimeArtifactHot {
        &mut self.hot
    }

    pub fn warm(&self) -> &RuntimeArtifactWarm {
        &self.warm
    }

    pub fn warm_mut(&mut self) -> &mut RuntimeArtifactWarm {
        &mut self.warm
    }

    pub fn output_hash(&self) -> StableHashValue {
        self.hot.output_hash
    }

    pub fn output_change(&self) -> OutputChange {
        self.hot.output_change
    }

    pub fn recomputed(&self) -> bool {
        self.hot.recomputed
    }

    pub fn dependency_count(&self) -> u32 {
        self.hot.dependency_count
    }

    pub fn meaningful_input_changes(&self) -> u32 {
        self.hot.meaningful_input_changes
    }

    pub fn changed_partition_count(&self) -> u32 {
        self.hot.changed_partition_count
    }

    pub fn propagation_suppressed(&self) -> bool {
        self.hot.propagation_suppressed
    }

    pub fn changed_scopes(&self) -> &CompactChangedScopeProof {
        &self.hot.changed_scopes
    }

    pub fn output_identity(&self) -> Option<&OutputIdentity> {
        self.warm.output_identity.as_ref()
    }

    pub fn continuity_token(&self) -> Option<&ArtifactContinuityToken> {
        self.warm.continuity_token.as_ref()
    }

    pub fn continuity_token_authority(&self) -> &ContinuityAuthorityToken {
        &self.warm.continuity_token
    }

    pub fn memoized_origin(&self) -> MemoizedResultOrigin {
        self.warm.memoized_origin
    }

    pub fn reuse_basis(&self) -> &ReuseOperationalBasis {
        &self.warm.reuse_basis
    }

    pub fn reuse_origin(&self) -> ReuseOrigin {
        self.warm.reuse_origin
    }

    pub fn reuse_boundary_authority(&self) -> Option<&ReuseBoundaryAuthority> {
        self.warm.reuse_boundary_authority.as_ref()
    }

    pub fn lineage_artifact_id(&self) -> ArtifactTransitionKey {
        self.warm.lineage_artifact_id
    }

    pub fn merge_authority(&self) -> &ArtifactMergeAuthority {
        &self.warm.merge_authority
    }

    pub fn set_lineage_artifact_id(&mut self, artifact_id: Option<LineageArtifactId>) {
        self.warm.lineage_artifact_id = ArtifactTransitionKey::new(artifact_id);
    }

    pub fn reuse_boundary_snapshot(&self) -> RuntimeArtifactReuseBoundarySnapshot {
        RuntimeArtifactReuseBoundarySnapshot {
            output_identity: self.output_identity().cloned(),
            continuity_token: self.continuity_token_authority().clone_inner(),
            reuse_boundary_authority: self.reuse_boundary_authority().cloned(),
        }
    }

    pub fn operational_summary(&self) -> RuntimeArtifactOperationalSummary {
        RuntimeArtifactOperationalSummary {
            memoized_origin: self.memoized_origin(),
            reuse_basis: self.reuse_basis().clone_inner(),
            reuse_origin: self.reuse_origin(),
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl RuntimeArtifactFinalizeImage {
    pub fn from_runtime_state(state: &RuntimeArtifactState) -> Self {
        Self {
            hot: state.hot().clone(),
            reuse_origin: state.reuse_origin(),
            reuse_boundary_authority: state.reuse_boundary_authority().cloned(),
            lineage_artifact_id: state.lineage_artifact_id(),
        }
    }

    pub fn output_change(&self) -> OutputChange {
        self.hot.output_change
    }

    pub fn recomputed(&self) -> bool {
        self.hot.recomputed
    }

    pub fn propagation_suppressed(&self) -> bool {
        self.hot.propagation_suppressed
    }

    pub fn reuse_origin(&self) -> ReuseOrigin {
        self.reuse_origin
    }

    pub fn reuse_boundary_authority(&self) -> Option<&ReuseBoundaryAuthority> {
        self.reuse_boundary_authority.as_ref()
    }

    pub fn changed_partition_count(&self) -> u32 {
        self.hot.changed_partition_count
    }

    pub fn lineage_artifact_id(&self) -> ArtifactTransitionKey {
        self.lineage_artifact_id
    }
}
