use worth_store_recovery_physics::PageLsn;

use crate::{
    epoch::{manifest_epoch_from_entry_seed, root_epoch_from_entry_seed},
    CheckpointPublicationRootBasis, CurrentPhysicalRootBasis, ManifestEpoch,
    ManifestLocatorRootBasis, RecoveryRootBasis, RootEpoch,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalIsolationEntryIdentity {
    recovered_root: String,
    admitted_page_lsn_frontier: Option<PageLsn>,
    source_decision_digest: String,
    replayed_frames: usize,
    source_candidate_count: usize,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIsolationRootEpochBasis {
    root_epoch: RootEpoch,
    manifest_epoch: ManifestEpoch,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

impl PhysicalIsolationEntryIdentity {
    #[cfg(feature = "certification-authority")]
    pub fn from_certification_boundary(
        recovered_root: &str,
        admitted_page_lsn_frontier: Option<PageLsn>,
        source_decision_digest: &str,
        replayed_frames: usize,
        source_candidate_count: usize,
        store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    ) -> Self {
        Self {
            recovered_root: recovered_root.to_string(),
            admitted_page_lsn_frontier,
            source_decision_digest: source_decision_digest.to_string(),
            replayed_frames,
            source_candidate_count,
            store_authority_identity,
        }
    }

    pub fn recovered_root(&self) -> &str {
        &self.recovered_root
    }

    pub const fn admitted_page_lsn_frontier(&self) -> Option<PageLsn> {
        self.admitted_page_lsn_frontier
    }

    pub fn source_decision_digest(&self) -> &str {
        &self.source_decision_digest
    }

    pub const fn replayed_frames(&self) -> usize {
        self.replayed_frames
    }

    pub const fn source_candidate_count(&self) -> usize {
        self.source_candidate_count
    }

    pub const fn store_authority_identity(
        &self,
    ) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.store_authority_identity
    }

    pub fn root_epoch_basis(&self) -> PhysicalIsolationRootEpochBasis {
        let seed = stable_identity_hash(self);
        PhysicalIsolationRootEpochBasis {
            root_epoch: root_epoch_from_entry_seed(seed),
            manifest_epoch: manifest_epoch_from_entry_seed(seed),
            store_authority_identity: self.store_authority_identity,
        }
    }

    #[cfg(feature = "certification-authority")]
    pub(crate) fn boundary_artifact_id(&self) -> u64 {
        stable_identity_hash(self)
    }
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

fn stable_identity_hash(identity: &PhysicalIsolationEntryIdentity) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    mix_bytes(&mut hash, identity.recovered_root.as_bytes());
    mix_u64(&mut hash, page_lsn_u64(identity.admitted_page_lsn_frontier));
    mix_bytes(&mut hash, identity.source_decision_digest.as_bytes());
    mix_u64(&mut hash, identity.replayed_frames as u64);
    mix_u64(&mut hash, identity.source_candidate_count as u64);
    mix_bytes(&mut hash, &identity.store_authority_identity.fingerprint());
    hash
}

fn page_lsn_u64(page_lsn: Option<PageLsn>) -> u64 {
    page_lsn.map_or(0, |lsn| lsn.lsn().get())
}

fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn mix_u64(hash: &mut u64, value: u64) {
    mix_bytes(hash, &value.to_le_bytes());
}
