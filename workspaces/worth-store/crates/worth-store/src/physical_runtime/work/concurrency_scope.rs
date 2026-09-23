use sha2::{Digest, Sha256};
use worth_store_physical_format::RecordFrameCoordinate;

use super::effect_footprint::{
    effect_relation, lower_effect_footprint, shares_coordination_identity, PhysicalEffectFootprint,
    PhysicalEffectRelation,
};
use super::PhysicalWorkIntent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWorkConcurrencyScope {
    digest: [u8; 32],
    security: worth_store_security::StoreSecurityScopeIdentity,
    coordinates: Box<[RecordFrameCoordinate]>,
    footprint: PhysicalEffectFootprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkConcurrencyRelation {
    DisjointArtifacts,
    SameArtifactDisjointRanges,
    Overlapping,
}

impl PhysicalWorkConcurrencyScope {
    pub(in crate::physical_runtime::work) fn derive(intent: &PhysicalWorkIntent) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth-store.physical-concurrency-scope.v1");
        digest.update(intent.identity().store().bytes());
        digest.update(intent.security().stable_fingerprint());
        digest.update(intent.scope().stable_digest());
        Self {
            digest: digest.finalize().into(),
            security: intent.security(),
            coordinates: intent.scope().coordinates().into(),
            footprint: lower_effect_footprint(intent),
        }
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub const fn security(&self) -> worth_store_security::StoreSecurityScopeIdentity {
        self.security
    }

    pub fn coordinates(&self) -> &[RecordFrameCoordinate] {
        &self.coordinates
    }

    pub fn relation(&self, other: &Self) -> PhysicalWorkConcurrencyRelation {
        match effect_relation(&self.footprint, &other.footprint) {
            PhysicalEffectRelation::Conflict | PhysicalEffectRelation::SharedRead => {
                PhysicalWorkConcurrencyRelation::Overlapping
            }
            PhysicalEffectRelation::Disjoint
                if shares_coordination_identity(&self.footprint, &other.footprint) =>
            {
                PhysicalWorkConcurrencyRelation::SameArtifactDisjointRanges
            }
            PhysicalEffectRelation::Disjoint => PhysicalWorkConcurrencyRelation::DisjointArtifacts,
        }
    }
}
