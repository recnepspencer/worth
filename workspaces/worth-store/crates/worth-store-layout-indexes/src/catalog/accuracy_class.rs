#[cfg(test)]
use super::ArtifactFamilyDenial;
use super::{ArtifactAuthorityRoleWitness, ArtifactFamilyAuthorityDisposition};
use worth_store_contracts::DurableArtifactFamilyId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedAccuracyClass {
    Exact,
    Conservative,
    Approximate,
    Heuristic,
    Advisory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactDerivedAccuracyWitness {
    role: ArtifactAuthorityRoleWitness,
    accuracy: DerivedAccuracyClass,
}

impl ArtifactDerivedAccuracyWitness {
    pub(crate) const fn new(
        role: ArtifactAuthorityRoleWitness,
        accuracy: DerivedAccuracyClass,
    ) -> Self {
        Self { role, accuracy }
    }

    pub const fn role(self) -> ArtifactAuthorityRoleWitness {
        self.role
    }

    pub const fn family_id(self) -> DurableArtifactFamilyId {
        self.role.family_id()
    }

    pub const fn accuracy(self) -> DerivedAccuracyClass {
        self.accuracy
    }
}

pub(crate) fn declare_derived_accuracy_class(
    role: ArtifactAuthorityRoleWitness,
) -> ArtifactDerivedAccuracyWitness {
    let accuracy = match role.classification().authority() {
        ArtifactFamilyAuthorityDisposition::Authoritative => DerivedAccuracyClass::Exact,
        ArtifactFamilyAuthorityDisposition::Diagnostic
        | ArtifactFamilyAuthorityDisposition::Terminal
        | ArtifactFamilyAuthorityDisposition::Certification => DerivedAccuracyClass::Advisory,
        ArtifactFamilyAuthorityDisposition::Derived => match role.family_id() {
            DurableArtifactFamilyId::ChunkTreeRoot
            | DurableArtifactFamilyId::DedupeIndex
            | DurableArtifactFamilyId::ReachabilityEdge
            | DurableArtifactFamilyId::DerivedRetentionLegacyLayoutMaterialization
            | DurableArtifactFamilyId::DerivedRetentionLegacyScopeSliceMembership
            | DurableArtifactFamilyId::DerivedRetentionLegacyStructuralBlock
            | DurableArtifactFamilyId::DerivedRetentionLegacyChunkMembership => {
                DerivedAccuracyClass::Exact
            }
            DurableArtifactFamilyId::LayoutCompactionUnit
            | DurableArtifactFamilyId::PlacementStableBasis
            | DurableArtifactFamilyId::PlacementSnapshotFamily
            | DurableArtifactFamilyId::PlacementBranchDeltaFamily
            | DurableArtifactFamilyId::PlacementLegacyLayoutFamily
            | DurableArtifactFamilyId::SchedulerReservationIndex
            | DurableArtifactFamilyId::TierPlacementManifest
            | DurableArtifactFamilyId::ColdRecallQueue
            | DurableArtifactFamilyId::RecallAmplificationIndex
            | DurableArtifactFamilyId::SupportSchema
            | DurableArtifactFamilyId::SupportLineage
            | DurableArtifactFamilyId::SupportCursor
            | DurableArtifactFamilyId::SupportEmbeddedCheckpoint => {
                DerivedAccuracyClass::Conservative
            }
            DurableArtifactFamilyId::MaintenanceSnapshot
            | DurableArtifactFamilyId::MaintenanceCompaction
            | DurableArtifactFamilyId::MaintenanceReclaim
            | DurableArtifactFamilyId::MaintenanceCapsule
            | DurableArtifactFamilyId::MaintenanceQueueDeclaration => {
                DerivedAccuracyClass::Heuristic
            }
            _ => DerivedAccuracyClass::Advisory,
        },
    };

    ArtifactDerivedAccuracyWitness::new(role, accuracy)
}

#[cfg(test)]
pub(crate) fn require_exact_accuracy_claim(
    accuracy: ArtifactDerivedAccuracyWitness,
) -> Result<ArtifactDerivedAccuracyWitness, ArtifactFamilyDenial> {
    match accuracy.accuracy() {
        DerivedAccuracyClass::Exact => Ok(accuracy),
        DerivedAccuracyClass::Conservative
        | DerivedAccuracyClass::Approximate
        | DerivedAccuracyClass::Heuristic
        | DerivedAccuracyClass::Advisory => {
            Err(ArtifactFamilyDenial::InexactFamilyCannotSatisfyExactClaim)
        }
    }
}
