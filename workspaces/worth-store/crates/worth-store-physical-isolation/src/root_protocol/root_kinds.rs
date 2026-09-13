use crate::{
    CurrentGenerationPhysicalReference, ExtentPublicationEpochBasis,
    FutureChunkPublicationEpochBasis, GenerationCountedReferenceDenial, ManifestEpoch,
    PagePublicationEpochBasis, PhysicalOrderingContract, PhysicalOrderingContractDenial,
    PhysicalOrderingSite, RootEpoch, SegmentPublicationEpochBasis,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentPhysicalRoot {
    epoch: RootEpoch,
    manifest_epoch: ManifestEpoch,
    ordering: PhysicalOrderingContract,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointPublicationRoot {
    epoch: RootEpoch,
    ordering: PhysicalOrderingContract,
    checkpoint_identity: CheckpointPublicationIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryRoot {
    epoch: RootEpoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestLocatorRoot {
    epoch: RootEpoch,
}

#[derive(Debug, Clone, Copy)]
pub struct CurrentPhysicalRootBasis {
    root_epoch: RootEpoch,
    manifest_epoch: ManifestEpoch,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

#[derive(Debug, Clone, Copy)]
pub struct CheckpointPublicationRootBasis {
    epoch: RootEpoch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointPublicationIdentity {
    checkpoint: PhysicalCheckpointIdentity,
}

#[derive(Debug, Clone, Copy)]
pub struct RecoveryRootBasis {
    epoch: RootEpoch,
}

#[derive(Debug, Clone, Copy)]
pub struct ManifestLocatorRootBasis {
    epoch: RootEpoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKindMismatchDenial {
    CheckpointPublicationRootCannotAdmitCurrentReadPlan,
    RecoveryRootRequiresEntryReadmission,
    ManifestLocatorRootCannotAdmitCurrentReadPlan,
}

impl CurrentPhysicalRoot {
    pub fn from_physical_isolation_entry(
        basis: CurrentPhysicalRootBasis,
        ordering: PhysicalOrderingContract,
    ) -> Result<Self, PhysicalOrderingContractDenial> {
        let ordering = ordering.require_site(PhysicalOrderingSite::RootSwap)?;
        Ok(Self {
            epoch: basis.root_epoch,
            manifest_epoch: basis.manifest_epoch,
            ordering,
            store_authority_identity: basis.store_authority_identity,
        })
    }

    pub const fn epoch(self) -> RootEpoch {
        self.epoch
    }

    pub const fn manifest_epoch(self) -> ManifestEpoch {
        self.manifest_epoch
    }

    pub const fn ordering(self) -> PhysicalOrderingContract {
        self.ordering
    }

    pub const fn scope(self) -> u64 {
        self.epoch.get()
    }

    pub const fn store_authority_identity(
        self,
    ) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.store_authority_identity
    }

    pub fn admit_segment_publication_epoch(
        self,
        reference: CurrentGenerationPhysicalReference,
    ) -> Result<SegmentPublicationEpochBasis, GenerationCountedReferenceDenial> {
        SegmentPublicationEpochBasis::admit(self.scope(), reference)
    }

    pub fn admit_extent_publication_epoch(
        self,
        reference: CurrentGenerationPhysicalReference,
    ) -> Result<ExtentPublicationEpochBasis, GenerationCountedReferenceDenial> {
        ExtentPublicationEpochBasis::admit(self.scope(), reference)
    }

    pub fn admit_page_publication_epoch(
        self,
        reference: CurrentGenerationPhysicalReference,
    ) -> Result<PagePublicationEpochBasis, GenerationCountedReferenceDenial> {
        PagePublicationEpochBasis::admit(self.scope(), reference)
    }

    pub const fn future_chunk_publication_epoch_placeholder(
        self,
    ) -> FutureChunkPublicationEpochBasis {
        FutureChunkPublicationEpochBasis::blob_placement_placeholder(self.scope())
    }
}

impl CheckpointPublicationRoot {
    pub fn from_checkpoint_publication(
        basis: CheckpointPublicationRootBasis,
        ordering: PhysicalOrderingContract,
        checkpoint_identity: CheckpointPublicationIdentity,
    ) -> Result<Self, PhysicalOrderingContractDenial> {
        let ordering = ordering.require_site(PhysicalOrderingSite::RootSwap)?;
        Ok(Self {
            epoch: basis.epoch,
            ordering,
            checkpoint_identity,
        })
    }

    pub const fn epoch(&self) -> RootEpoch {
        self.epoch
    }

    pub const fn ordering(&self) -> PhysicalOrderingContract {
        self.ordering
    }

    pub const fn checkpoint_identity(&self) -> &CheckpointPublicationIdentity {
        &self.checkpoint_identity
    }
}

impl CheckpointPublicationIdentity {
    pub const fn from_physical_checkpoint_identity(checkpoint: PhysicalCheckpointIdentity) -> Self {
        Self { checkpoint }
    }

    pub fn matches_physical_checkpoint_identity(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
    ) -> bool {
        self.checkpoint == checkpoint
    }

    pub const fn physical_checkpoint_identity(&self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }
}

impl RecoveryRoot {
    pub const fn from_recovery_basis(basis: RecoveryRootBasis) -> Self {
        Self { epoch: basis.epoch }
    }

    pub const fn epoch(self) -> RootEpoch {
        self.epoch
    }
}

impl ManifestLocatorRoot {
    pub const fn from_manifest_locator_basis(basis: ManifestLocatorRootBasis) -> Self {
        Self { epoch: basis.epoch }
    }

    pub const fn epoch(self) -> RootEpoch {
        self.epoch
    }
}

impl CurrentPhysicalRootBasis {
    pub(crate) const fn new(
        root_epoch: RootEpoch,
        manifest_epoch: ManifestEpoch,
        store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    ) -> Self {
        Self {
            root_epoch,
            manifest_epoch,
            store_authority_identity,
        }
    }

    pub const fn root_epoch(self) -> RootEpoch {
        self.root_epoch
    }

    pub const fn manifest_epoch(self) -> ManifestEpoch {
        self.manifest_epoch
    }

    pub const fn store_authority_identity(
        self,
    ) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.store_authority_identity
    }
}

impl CheckpointPublicationRootBasis {
    pub(crate) const fn new(epoch: RootEpoch) -> Self {
        Self { epoch }
    }

    pub const fn epoch(self) -> RootEpoch {
        self.epoch
    }
}

impl RecoveryRootBasis {
    pub(crate) const fn new(epoch: RootEpoch) -> Self {
        Self { epoch }
    }

    pub const fn epoch(self) -> RootEpoch {
        self.epoch
    }
}

impl ManifestLocatorRootBasis {
    pub(crate) const fn new(epoch: RootEpoch) -> Self {
        Self { epoch }
    }

    pub const fn epoch(self) -> RootEpoch {
        self.epoch
    }
}
