use super::canonical_encoding::CanonicalBindingEncoding;

use super::attempt_binding::AllocatedPhysicalMutationAttemptBinding;
use super::registry::PhysicalMutationUnresolvedBindingObservation;

mod decoding;
#[cfg(test)]
pub(super) mod tests;
pub(in crate::physical_runtime) use decoding::{
    decode_binding_basis, CanonicalBindingCursor, PhysicalBindingDecodingContext,
    PhysicalPersistedBindingDecodeDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PersistedPhysicalMutationAttemptBinding {
    key: super::PhysicalMutationIdempotencyKey,
    fingerprint: crate::physical_runtime::PhysicalMutationRequestFingerprint,
    mutation: crate::physical_runtime::PhysicalMutationIdentity,
    group: crate::physical_runtime::PhysicalDurabilityGroupMemberBinding,
    member: crate::physical_runtime::PhysicalWalMemberBasis,
    redo_digest: [u8; 32],
    bytes: Box<[u8]>,
}

impl PersistedPhysicalMutationAttemptBinding {
    pub(in crate::physical_runtime) fn from_allocated(
        binding: &AllocatedPhysicalMutationAttemptBinding,
    ) -> Self {
        let mut persisted = Self {
            key: binding.key().clone(),
            fingerprint: binding.fingerprint(),
            mutation: binding.mutation_identity(),
            group: binding.group_binding(),
            member: binding.member(),
            redo_digest: binding.redo_digest(),
            bytes: Box::default(),
        };
        persisted.bytes = persisted.encode().into_boxed_slice();
        debug_assert_eq!(persisted.bytes(), binding.encode_persisted());
        persisted
    }

    pub(in crate::physical_runtime) const fn observation(
        &self,
    ) -> PhysicalMutationUnresolvedBindingObservation {
        PhysicalMutationUnresolvedBindingObservation::new(
            self.key.identity(),
            self.fingerprint,
            self.mutation,
        )
    }

    pub(in crate::physical_runtime) const fn group(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityGroupMemberBinding {
        self.group
    }

    pub(in crate::physical_runtime) const fn member(
        &self,
    ) -> crate::physical_runtime::PhysicalWalMemberBasis {
        self.member
    }

    pub(in crate::physical_runtime) const fn idempotency_identity(
        &self,
    ) -> super::PhysicalMutationIdempotencyKeyIdentity {
        self.key.identity()
    }

    pub(in crate::physical_runtime) const fn policy_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityPolicyIdentity {
        self.key.lease().policy_identity()
    }

    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::physical_runtime) fn key(&self) -> &super::PhysicalMutationIdempotencyKey {
        &self.key
    }

    pub(in crate::physical_runtime) const fn fingerprint(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub(in crate::physical_runtime) const fn mutation(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationIdentity {
        self.mutation
    }

    pub(in crate::physical_runtime) const fn redo_digest(&self) -> [u8; 32] {
        self.redo_digest
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(320);
        self.encode_into(&mut bytes);
        bytes
    }

    fn encode_into(&self, bytes: &mut impl CanonicalBindingEncoding) {
        let lease = self.key.lease();
        let range = self.member.lsn_range();
        bytes.field(worth_store_wal::PHYSICAL_MUTATION_ATTEMPT_BINDING_DOMAIN);
        bytes.field(&self.key.identity().bytes());
        bytes.field(&lease.store_identity().bytes());
        bytes.field(&lease.policy_identity().bytes());
        bytes.write(&lease.issuance_generation().get().to_le_bytes());
        bytes.write(&lease.expiry_generation().get().to_le_bytes());
        bytes.field(&self.key.caller_material().bytes());
        bytes.field(&self.fingerprint.bytes());
        bytes.field(&self.mutation.store_identity().bytes());
        bytes.write(&self.mutation.runtime_identity().get().to_le_bytes());
        bytes.write(&self.mutation.lifecycle_generation().to_le_bytes());
        bytes.write(&self.mutation.operation_identity().get().to_le_bytes());
        bytes.field(&self.group.group_identity().bytes());
        bytes.write(&self.group.ordinal().get().to_le_bytes());
        bytes.write(&self.group.member_count().get().to_le_bytes());
        bytes.field(&self.group.membership_digest());
        bytes.field(&self.member.member_identity().bytes());
        bytes.write(&range.start().get().to_le_bytes());
        bytes.write(&range.end_exclusive().get().to_le_bytes());
        bytes.field(&self.redo_digest);
    }
}
