use serde::{Deserialize, Serialize};

mod retained_charge;

use super::identity::{ArtifactFamilyId, PersistentCorrespondenceKind};
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::core_profile::StableHashValue;
use crate::data::dependency::DependencySnapshotId;
use crate::data::performance::AuthorityPolicy;

/// Compact hot-lane authority for deterministic reuse/replay decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReuseBoundaryAuthority {
    pub topology_regime: u32,
    pub tolerance_regime: VersionComparatorPolicy,
    pub semantic_region_digest: StableHashValue,
    pub authority_policy: AuthorityPolicy,
    #[serde(default)]
    pub artifact_family: Option<ArtifactFamilyId>,
    #[serde(default)]
    pub structural_dependency_basis: DependencySnapshotId,
    #[serde(default)]
    pub partition_region_basis_digest: StableHashValue,
    #[serde(default)]
    pub partition_region_basis_count: u32,
    #[serde(default)]
    pub strategy_detail: ReuseStrategyBoundaryAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReuseStrategyBoundaryAuthority {
    #[default]
    None,
    CrossIdentity {
        persistent_correspondence_kind: PersistentCorrespondenceKind,
        persistent_correspondence_digest: StableHashValue,
        persistent_correspondence_valid: bool,
    },
    PartialArtifactSplice {
        composition_region_digest: StableHashValue,
        composition_region_count: u32,
    },
}

impl ReuseBoundaryAuthority {
    pub fn persistent_correspondence_kind(&self) -> Option<PersistentCorrespondenceKind> {
        match self.strategy_detail {
            ReuseStrategyBoundaryAuthority::CrossIdentity {
                persistent_correspondence_kind,
                ..
            } => Some(persistent_correspondence_kind),
            ReuseStrategyBoundaryAuthority::None
            | ReuseStrategyBoundaryAuthority::PartialArtifactSplice { .. } => None,
        }
    }

    pub fn composition_region_count(&self) -> u32 {
        match self.strategy_detail {
            ReuseStrategyBoundaryAuthority::PartialArtifactSplice {
                composition_region_count,
                ..
            } => composition_region_count,
            ReuseStrategyBoundaryAuthority::None
            | ReuseStrategyBoundaryAuthority::CrossIdentity { .. } => 0,
        }
    }
}
