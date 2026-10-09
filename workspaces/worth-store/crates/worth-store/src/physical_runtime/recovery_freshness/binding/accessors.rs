use super::*;

impl StoreRecoveryBindingFreshnessSample {
    /// Owned recovery-data backing, excluding this inline value and past scratch.
    /// Canonical redo and copy frames are distinct retained allocations.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = self.roster_heap_bytes()?;
        for member in &self.wal_members {
            bytes = bytes.checked_add(u64::try_from(member.canonical_redo.capacity()).ok()?)?;
        }
        for (_, frame) in &self.extent_copy_frames {
            bytes = bytes.checked_add(u64::try_from(frame.capacity()).ok()?)?;
        }
        Some(bytes)
    }

    pub(in crate::physical_runtime) fn roster_heap_bytes(&self) -> Option<u64> {
        fn bytes<T>(vector: &Vec<T>) -> Option<u64> {
            u64::try_from(vector.capacity())
                .ok()?
                .checked_mul(std::mem::size_of::<T>() as u64)
        }
        bytes(&self.operations)?
            .checked_add(bytes(&self.wal_members)?)?
            .checked_add(bytes(&self.retirements)?)?
            .checked_add(bytes(&self.release_intents)?)?
            .checked_add(bytes(&self.extent_copy_frames)?)?
            .checked_add(bytes(&self.blob_manifest_residue_cleanups)?)
    }

    pub fn charged_bytes(&self) -> u64 {
        self.backing.bytes()
    }

    pub const fn tier_epoch_activation(&self) -> Option<StoreTierEpochActivationObservation> {
        self.tier_epoch_activation
    }
    pub fn extent_copy_frames(&self) -> impl ExactSizeIterator<Item = (WalLsnRange, &[u8])> {
        self.extent_copy_frames
            .iter()
            .map(|(range, bytes)| (*range, bytes.as_slice()))
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
    /// Every sampled retirement-release intent, including resolved ones and
    /// those below the checkpoint cutoff; data for the ordered root history.
    pub fn release_intents(&self) -> &[worth_store_recovery_physics::RetirementReleaseIntent] {
        &self.release_intents
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
