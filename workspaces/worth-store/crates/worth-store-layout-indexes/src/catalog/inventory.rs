use super::{artifact_family_inventory_rows, PhysicalArtifactFamilyDeclaration};
use crate::ArtifactFamilyDenial;
use worth_store_contracts::{
    DerivedFamilyRetentionPolicy, DurableArtifactFamilyId, LayoutFamilyCompactionUnit,
    MaintenanceArtifactFamily, PlacementArtifactFamily, PublicationFamily, SupportArtifactFamily,
    WalRecordFamily,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactFamilyInventoryRow {
    declaration: PhysicalArtifactFamilyDeclaration,
}

impl ArtifactFamilyInventoryRow {
    pub(crate) const fn new(declaration: PhysicalArtifactFamilyDeclaration) -> Self {
        Self { declaration }
    }

    pub const fn declaration(&self) -> &PhysicalArtifactFamilyDeclaration {
        &self.declaration
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactFamilyInventory;

pub trait ExistingArtifactFamilySurface: private::Sealed {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId;
}

mod private {
    pub trait Sealed {}
}

impl ArtifactFamilyInventory {
    pub const fn current() -> Self {
        Self
    }

    pub fn rows(&self) -> &'static [ArtifactFamilyInventoryRow] {
        artifact_family_inventory_rows()
    }

    pub fn declaration(
        &self,
        family_id: DurableArtifactFamilyId,
    ) -> Result<&'static PhysicalArtifactFamilyDeclaration, ArtifactFamilyDenial> {
        declaration_in_rows(artifact_family_inventory_rows(), family_id)
    }

    pub fn admit_existing_family(
        &self,
        family: &impl ExistingArtifactFamilySurface,
    ) -> Result<&'static PhysicalArtifactFamilyDeclaration, ArtifactFamilyDenial> {
        self.declaration(family.canonical_family_id())
    }
}

pub(crate) fn declaration_in_rows(
    rows: &[ArtifactFamilyInventoryRow],
    family_id: DurableArtifactFamilyId,
) -> Result<&PhysicalArtifactFamilyDeclaration, ArtifactFamilyDenial> {
    let mut index = 0;
    while index < rows.len() {
        let row = &rows[index];
        if row.declaration().family_id() == family_id {
            return Ok(row.declaration());
        }
        index += 1;
    }
    Err(ArtifactFamilyDenial::MissingFamilyDeclaration)
}

impl private::Sealed for WalRecordFamily {}
impl ExistingArtifactFamilySurface for WalRecordFamily {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self {
            Self::DurableMutationIntent => DurableArtifactFamilyId::WalDurableMutationIntent,
            Self::HostedRuntimeCommitResult => {
                DurableArtifactFamilyId::WalHostedRuntimeCommitResult
            }
            Self::BulkCheckpointPublicationIntent => {
                DurableArtifactFamilyId::WalBulkCheckpointPublicationIntent
            }
            Self::DurablePublicationProgress => {
                DurableArtifactFamilyId::WalDurablePublicationProgress
            }
            Self::RecoveryDecision => DurableArtifactFamilyId::WalRecoveryDecision,
        }
    }
}

impl private::Sealed for MaintenanceArtifactFamily {}
impl ExistingArtifactFamilySurface for MaintenanceArtifactFamily {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self {
            Self::Snapshot => DurableArtifactFamilyId::MaintenanceSnapshot,
            Self::Compaction => DurableArtifactFamilyId::MaintenanceCompaction,
            Self::Reclaim => DurableArtifactFamilyId::MaintenanceReclaim,
            Self::Capsule => DurableArtifactFamilyId::MaintenanceCapsule,
        }
    }
}

impl private::Sealed for SupportArtifactFamily {}
impl ExistingArtifactFamilySurface for SupportArtifactFamily {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self {
            Self::SchemaSupport => DurableArtifactFamilyId::SupportSchema,
            Self::LineageSupport => DurableArtifactFamilyId::SupportLineage,
            Self::CursorSupport => DurableArtifactFamilyId::SupportCursor,
            Self::EmbeddedCheckpoint => DurableArtifactFamilyId::SupportEmbeddedCheckpoint,
        }
    }
}

impl private::Sealed for PlacementArtifactFamily {}
impl ExistingArtifactFamilySurface for PlacementArtifactFamily {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self {
            Self::AuthoritativeBranchHead => {
                DurableArtifactFamilyId::PlacementAuthoritativeBranchHead
            }
            Self::RetainedAuthority => DurableArtifactFamilyId::PlacementRetainedAuthority,
            Self::StableBasis => DurableArtifactFamilyId::PlacementStableBasis,
            Self::SnapshotFamily => DurableArtifactFamilyId::PlacementSnapshotFamily,
            Self::BranchDeltaFamily => DurableArtifactFamilyId::PlacementBranchDeltaFamily,
            Self::LegacyLayoutFamily => DurableArtifactFamilyId::PlacementLegacyLayoutFamily,
        }
    }
}

impl private::Sealed for PublicationFamily {}
impl ExistingArtifactFamilySurface for PublicationFamily {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self {
            Self::WalIntent => DurableArtifactFamilyId::PublicationWalIntent,
            Self::WalCanonicalResult => DurableArtifactFamilyId::PublicationWalCanonicalResult,
            Self::WalPublicationProgress => {
                DurableArtifactFamilyId::PublicationWalPublicationProgress
            }
            Self::AuthoritativeCommitAppendUnit => {
                DurableArtifactFamilyId::PublicationAuthoritativeCommitAppendUnit
            }
            Self::BranchHeadPublication => {
                DurableArtifactFamilyId::PublicationBranchHeadPublication
            }
            Self::AcknowledgmentEligibility => {
                DurableArtifactFamilyId::PublicationAcknowledgmentEligibility
            }
            Self::SnapshotBasis => DurableArtifactFamilyId::PublicationSnapshotBasis,
            Self::SnapshotImage => DurableArtifactFamilyId::PublicationSnapshotImage,
        }
    }
}

impl private::Sealed for DerivedFamilyRetentionPolicy {}
impl ExistingArtifactFamilySurface for DerivedFamilyRetentionPolicy {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self {
            Self::LegacyLayoutMaterialization => {
                DurableArtifactFamilyId::DerivedRetentionLegacyLayoutMaterialization
            }
            Self::LegacyScopeSliceMembership => {
                DurableArtifactFamilyId::DerivedRetentionLegacyScopeSliceMembership
            }
            Self::LegacyStructuralBlock => {
                DurableArtifactFamilyId::DerivedRetentionLegacyStructuralBlock
            }
            Self::LegacyChunkMembership => {
                DurableArtifactFamilyId::DerivedRetentionLegacyChunkMembership
            }
        }
    }
}

impl private::Sealed for LayoutFamilyCompactionUnit {}
impl ExistingArtifactFamilySurface for LayoutFamilyCompactionUnit {
    fn canonical_family_id(&self) -> DurableArtifactFamilyId {
        match self.family_kind() {
            worth_store_contracts::LayoutCompactionFamilyKind::LayoutCompactionUnit => {
                DurableArtifactFamilyId::LayoutCompactionUnit
            }
        }
    }
}
