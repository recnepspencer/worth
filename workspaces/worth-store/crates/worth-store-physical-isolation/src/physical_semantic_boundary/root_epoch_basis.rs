use crate::{
    CheckpointPublicationRootBasis, CurrentPhysicalRootBasis, ManifestEpoch,
    ManifestLocatorRootBasis, RecoveryRootBasis, RootEpoch,
};

/// Descriptive correlation projected from an already-issued physical root.
/// Recovery reports and copied entry fields cannot construct this basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIsolationRootEpochBasis {
    root_epoch: RootEpoch,
    manifest_epoch: ManifestEpoch,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

impl PhysicalIsolationRootEpochBasis {
    pub(crate) const fn from_current_root(root: crate::CurrentPhysicalRoot) -> Self {
        Self {
            root_epoch: root.epoch(),
            manifest_epoch: root.manifest_epoch(),
            store_authority_identity: root.store_authority_identity(),
        }
    }

    pub const fn epoch(&self) -> RootEpoch {
        self.root_epoch
    }

    pub const fn manifest_epoch(&self) -> ManifestEpoch {
        self.manifest_epoch
    }

    pub const fn current_root_basis(&self) -> CurrentPhysicalRootBasis {
        CurrentPhysicalRootBasis::new(
            self.root_epoch,
            self.manifest_epoch,
            self.store_authority_identity,
        )
    }

    pub const fn checkpoint_publication_root_basis(&self) -> CheckpointPublicationRootBasis {
        CheckpointPublicationRootBasis::new(self.root_epoch)
    }

    pub const fn recovery_root_basis(&self) -> RecoveryRootBasis {
        RecoveryRootBasis::new(self.root_epoch)
    }

    pub const fn manifest_locator_root_basis(&self) -> ManifestLocatorRootBasis {
        ManifestLocatorRootBasis::new(self.root_epoch)
    }
}
