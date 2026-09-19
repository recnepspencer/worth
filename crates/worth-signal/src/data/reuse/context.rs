mod authority;
mod digest;
mod identity;

use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::comparator::VersionComparatorPolicy;
use crate::data::dependency::DependencySnapshotId;
use crate::data::performance::AuthorityPolicy;
use crate::data::proof::PartitionScopeSet;

pub use authority::{ReuseBoundaryAuthority, ReuseStrategyBoundaryAuthority};
pub(crate) use digest::{
    stable_partition_scope_digest_from_slice, stable_persistent_correspondence_digest,
    stable_semantic_region_digest_from_parts,
};
pub use identity::{
    ArtifactFamilyId, PersistentCorrespondenceEvidence, PersistentCorrespondenceKind,
    ReuseSemanticRegionIdentity, ReuseStrategyBoundaryContext,
};

/// Compact runtime evidence needed to certify artifact reuse across semantic boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseBoundaryContext {
    pub topology_regime: u32,
    pub tolerance_regime: VersionComparatorPolicy,
    pub semantic_region: ReuseSemanticRegionIdentity,
    pub authority_policy: AuthorityPolicy,
    #[serde(default)]
    pub artifact_family: Option<ArtifactFamilyId>,
    #[serde(default)]
    pub structural_dependency_basis: DependencySnapshotId,
    #[serde(default)]
    pub partition_region_basis: PartitionScopeSet,
    #[serde(default)]
    pub strategy_detail: ReuseStrategyBoundaryContext,
}

impl ReuseBoundaryContext {
    pub fn persistent_correspondence(&self) -> Option<&PersistentCorrespondenceEvidence> {
        match &self.strategy_detail {
            ReuseStrategyBoundaryContext::CrossIdentity {
                persistent_correspondence,
            } => Some(persistent_correspondence),
            ReuseStrategyBoundaryContext::None
            | ReuseStrategyBoundaryContext::PartialArtifactSplice { .. } => None,
        }
    }

    pub fn composition_regions(&self) -> Option<&PartitionScopeSet> {
        match &self.strategy_detail {
            ReuseStrategyBoundaryContext::PartialArtifactSplice {
                composition_regions,
            } => Some(composition_regions),
            ReuseStrategyBoundaryContext::None
            | ReuseStrategyBoundaryContext::CrossIdentity { .. } => None,
        }
    }

    pub fn authority(&self) -> ReuseBoundaryAuthority {
        ReuseBoundaryAuthority {
            topology_regime: self.topology_regime,
            tolerance_regime: self.tolerance_regime.clone(),
            semantic_region_digest: digest::stable_semantic_region_digest(&self.semantic_region),
            authority_policy: self.authority_policy,
            artifact_family: self.artifact_family.clone(),
            structural_dependency_basis: self.structural_dependency_basis,
            partition_region_basis_digest: digest::stable_partition_scope_digest(
                &self.partition_region_basis,
            ),
            partition_region_basis_count: self.partition_region_basis.len() as u32,
            strategy_detail: match &self.strategy_detail {
                ReuseStrategyBoundaryContext::None => ReuseStrategyBoundaryAuthority::None,
                ReuseStrategyBoundaryContext::CrossIdentity {
                    persistent_correspondence,
                } => ReuseStrategyBoundaryAuthority::CrossIdentity {
                    persistent_correspondence_kind: persistent_correspondence.kind(),
                    persistent_correspondence_digest:
                        digest::stable_persistent_correspondence_digest(persistent_correspondence),
                    persistent_correspondence_valid: persistent_correspondence
                        .is_structurally_valid(),
                },
                ReuseStrategyBoundaryContext::PartialArtifactSplice {
                    composition_regions,
                } => ReuseStrategyBoundaryAuthority::PartialArtifactSplice {
                    composition_region_digest: digest::stable_partition_scope_digest(
                        composition_regions,
                    ),
                    composition_region_count: composition_regions.len() as u32,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::proof::PartitionScopeSet;

    #[test]
    fn strategy_detail_accessors_are_mutually_exclusive() {
        let cross_identity = ReuseBoundaryContext {
            topology_regime: 1,
            tolerance_regime: VersionComparatorPolicy::Exact,
            semantic_region: ReuseSemanticRegionIdentity::new(
                crate::data::handle::NodeId::new(1, 0),
                false,
                Vec::new(),
                crate::data::node::ContextRequirement::None,
            ),
            authority_policy: AuthorityPolicy::SpeculativeThenReconcile,
            artifact_family: None,
            structural_dependency_basis: crate::data::dependency::DependencySnapshotId::EMPTY,
            partition_region_basis: PartitionScopeSet::default(),
            strategy_detail: ReuseStrategyBoundaryContext::CrossIdentity {
                persistent_correspondence: PersistentCorrespondenceEvidence::host_supplied_key(
                    "mesh-001",
                ),
            },
        };
        assert!(cross_identity.persistent_correspondence().is_some());
        assert!(cross_identity.composition_regions().is_none());

        let partial_splice = ReuseBoundaryContext {
            strategy_detail: ReuseStrategyBoundaryContext::PartialArtifactSplice {
                composition_regions: PartitionScopeSet::new([
                    crate::data::output::PartitionSubscription::whole_partition("wing"),
                ]),
            },
            ..cross_identity.clone()
        };
        assert!(partial_splice.persistent_correspondence().is_none());
        assert_eq!(
            partial_splice
                .composition_regions()
                .map(|regions| regions.len()),
            Some(1)
        );
    }
}
