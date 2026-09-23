use super::*;

impl StoreRecoveryBindingFreshnessSample {
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
