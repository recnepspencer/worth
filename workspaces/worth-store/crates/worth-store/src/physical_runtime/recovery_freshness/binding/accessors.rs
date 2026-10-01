use super::*;

impl StoreRecoveryBindingFreshnessSample {
    /// Owned recovery-data backing, excluding this inline value and past scratch.
    /// Canonical redo and copy frames are distinct retained allocations.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let storage = [
            std::mem::size_of_val(&*self.operations),
            std::mem::size_of_val(&*self.wal_members),
            std::mem::size_of_val(&*self.retirements),
            std::mem::size_of_val(&*self.extent_copy_frames),
            std::mem::size_of_val(&*self.blob_manifest_residue_cleanups),
        ];
        let mut bytes = storage.into_iter().try_fold(0_u64, |total, size| {
            total.checked_add(u64::try_from(size).ok()?)
        })?;
        for member in &self.wal_members {
            bytes = bytes.checked_add(u64::try_from(member.canonical_redo.len()).ok()?)?;
        }
        for (_, frame) in &self.extent_copy_frames {
            bytes = bytes.checked_add(u64::try_from(frame.len()).ok()?)?;
        }
        Some(bytes)
    }

    pub const fn tier_epoch_activation(&self) -> Option<StoreTierEpochActivationObservation> {
        self.tier_epoch_activation
    }
    pub fn extent_copy_frames(&self) -> &[(WalLsnRange, Box<[u8]>)] {
        &self.extent_copy_frames
    }
    pub fn blob_manifest_residue_cleanups(
        &self,
    ) -> &[(
        WalLsnRange,
        worth_store_physical_format::BlobManifestResidueCleanup,
        bool,
    )] {
        &self.blob_manifest_residue_cleanups
    }
    pub const fn manifest_cleanup_sampling_peak_bytes(&self) -> u64 {
        self.manifest_cleanup_sampling_peak_bytes
    }
    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn selected_checkpoint_generation(&self) -> u64 {
        self.selected_checkpoint_generation
    }
    pub const fn sealed_basis_identity(&self) -> [u8; 32] {
        self.sealed_basis_identity
    }
    pub const fn policy_identity(&self) -> [u8; 32] {
        self.policy_identity
    }
    pub fn operations(&self) -> &[StoreRecoveryOperationEvidence] {
        &self.operations
    }
    pub fn wal_members(&self) -> &[StoreRecoveryWalMember] {
        &self.wal_members
    }
    pub fn retirements(&self) -> &[StoreRecoveryRetirementObligation] {
        &self.retirements
    }
}

impl StoreRecoveryOperationEvidence {
    pub const fn idempotency_identity(&self) -> [u8; 32] {
        self.idempotency_identity
    }
    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }
    pub const fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.request_fingerprint
    }
    pub const fn lease_issuance_generation(&self) -> u64 {
        self.lease_issuance_generation
    }
    pub const fn lease_expiry_generation(&self) -> u64 {
        self.lease_expiry_generation
    }
    pub const fn freshness(&self) -> StoreRecoveryBindingFreshness {
        self.freshness
    }
    pub const fn fate(&self) -> StoreRecoveryOperationFate {
        self.fate
    }
    pub const fn attempt_binding_identity(&self) -> Option<[u8; 32]> {
        self.attempt_binding_identity
    }
}

impl StoreRecoveryWalMember {
    pub const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }
    pub const fn operation_identity(&self) -> [u8; 32] {
        self.operation_identity
    }
    pub const fn group_identity(&self) -> [u8; 32] {
        self.group_identity
    }
    pub const fn group_member_identity(&self) -> [u8; 32] {
        self.group_member_identity
    }
    pub const fn group_member_ordinal(&self) -> u32 {
        self.group_member_ordinal
    }
    pub const fn group_member_count(&self) -> u32 {
        self.group_member_count
    }
    pub const fn group_membership_digest(&self) -> [u8; 32] {
        self.group_membership_digest
    }
    pub fn canonical_redo(&self) -> &[u8] {
        &self.canonical_redo
    }
}
