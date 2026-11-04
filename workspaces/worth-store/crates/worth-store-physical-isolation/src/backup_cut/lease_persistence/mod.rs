mod binary_codec;
mod protected_owner;
#[cfg(test)]
mod tests;

use worth_foundational::CanonicalDigestId;
use worth_store_physical_format::PhysicalGenerationOwner;

use self::binary_codec::{canonicalize_protected_owners, decode, encode};
use super::BackupCutManifest;

pub(crate) use protected_owner::BackupProtectedPhysicalOwner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReachabilityLeasePersistenceRecord {
    cut_identity: CanonicalDigestId,
    protected: Vec<BackupProtectedPhysicalOwner>,
    encoded: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupReachabilityLeaseRecoveryDenial {
    InvalidEncoding,
    EmptyProtection,
    DuplicateProtection,
    NonCanonicalProtectionOrder,
    ProtectionCountOverflow,
    RecordLengthOverflow,
    AllocationFailed,
    InvalidOwnerCoordinate,
}

impl BackupReachabilityLeasePersistenceRecord {
    pub(crate) fn from_manifest(
        cut_identity: [u8; 32],
        manifest: &BackupCutManifest,
    ) -> Result<Self, BackupReachabilityLeaseRecoveryDenial> {
        let artifacts = manifest.artifacts();
        if artifacts.len() > u32::MAX as usize {
            return Err(BackupReachabilityLeaseRecoveryDenial::ProtectionCountOverflow);
        }
        let mut protected = Vec::new();
        protected
            .try_reserve_exact(artifacts.len())
            .map_err(|_| BackupReachabilityLeaseRecoveryDenial::AllocationFailed)?;
        protected.extend(artifacts.iter().map(|artifact| {
            BackupProtectedPhysicalOwner::from_owner(artifact.reclaim_reference().owner())
        }));
        Self::from_protected_owners(cut_identity, protected)
    }

    pub(super) fn from_protected_owners(
        cut_identity: [u8; 32],
        mut protected: Vec<BackupProtectedPhysicalOwner>,
    ) -> Result<Self, BackupReachabilityLeaseRecoveryDenial> {
        canonicalize_protected_owners(&mut protected)?;
        let encoded = encode(cut_identity, &protected)?;
        Ok(Self {
            cut_identity: CanonicalDigestId::new(cut_identity),
            protected,
            encoded,
        })
    }

    pub(crate) fn from_recovery_owners(
        cut_identity: [u8; 32],
        owners: impl IntoIterator<Item = Option<PhysicalGenerationOwner>>,
    ) -> Result<Self, BackupReachabilityLeaseRecoveryDenial> {
        let owners = owners.into_iter();
        let mut protected = Vec::new();
        let initial_capacity = owners.size_hint().0.min(u32::MAX as usize);
        protected
            .try_reserve_exact(initial_capacity)
            .map_err(|_| BackupReachabilityLeaseRecoveryDenial::AllocationFailed)?;
        for owner in owners {
            if protected.len() == u32::MAX as usize {
                return Err(BackupReachabilityLeaseRecoveryDenial::ProtectionCountOverflow);
            }
            if protected.len() == protected.capacity() {
                protected
                    .try_reserve(1)
                    .map_err(|_| BackupReachabilityLeaseRecoveryDenial::AllocationFailed)?;
            }
            protected.push(
                owner
                    .map(BackupProtectedPhysicalOwner::from_owner)
                    .ok_or(BackupReachabilityLeaseRecoveryDenial::InvalidOwnerCoordinate)?,
            );
        }
        Self::from_protected_owners(cut_identity, protected)
    }

    pub fn recover(encoded: &[u8]) -> Result<Self, BackupReachabilityLeaseRecoveryDenial> {
        let (cut_identity, protected) = decode(encoded)?;
        let mut durable_bytes = Vec::new();
        durable_bytes
            .try_reserve_exact(encoded.len())
            .map_err(|_| BackupReachabilityLeaseRecoveryDenial::AllocationFailed)?;
        durable_bytes.extend_from_slice(encoded);
        Ok(Self {
            cut_identity: CanonicalDigestId::new(cut_identity),
            protected,
            encoded: durable_bytes,
        })
    }

    pub const fn cut_identity(&self) -> [u8; 32] {
        *self.cut_identity.bytes()
    }

    pub fn protected_artifacts(&self) -> usize {
        self.protected.len()
    }

    pub fn recovery_bytes(&self) -> &[u8] {
        &self.encoded
    }

    pub(crate) fn protection(&self) -> &[BackupProtectedPhysicalOwner] {
        &self.protected
    }
}
