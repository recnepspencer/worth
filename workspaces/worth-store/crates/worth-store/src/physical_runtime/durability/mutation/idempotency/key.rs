use sha2::{Digest, Sha256};

use super::lease::PhysicalMutationIdempotencyLease;

const KEY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhysicalMutationIdempotencyMaterial([u8; 32]);

impl PhysicalMutationIdempotencyMaterial {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhysicalMutationIdempotencyKeyIdentity([u8; 32]);

impl PhysicalMutationIdempotencyKeyIdentity {
    pub(in crate::physical_runtime::durability::mutation::idempotency) const fn from_observed_bytes(
        bytes: [u8; 32],
    ) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMutationIdempotencyKey {
    identity: PhysicalMutationIdempotencyKeyIdentity,
    lease: PhysicalMutationIdempotencyLease,
    material: PhysicalMutationIdempotencyMaterial,
}

impl PhysicalMutationIdempotencyKey {
    pub(super) fn issue(
        lease: PhysicalMutationIdempotencyLease,
        material: PhysicalMutationIdempotencyMaterial,
    ) -> Self {
        let mut digest = Sha256::new();
        digest.update((KEY_DOMAIN.len() as u64).to_le_bytes());
        digest.update(KEY_DOMAIN);
        digest.update(lease.store_identity().bytes());
        digest.update(lease.policy_identity().bytes());
        digest.update(lease.issuance_generation().get().to_le_bytes());
        digest.update(lease.expiry_generation().get().to_le_bytes());
        digest.update(material.bytes());
        Self {
            identity: PhysicalMutationIdempotencyKeyIdentity(digest.finalize().into()),
            lease,
            material,
        }
    }

    pub const fn identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.identity
    }

    pub const fn lease(&self) -> PhysicalMutationIdempotencyLease {
        self.lease
    }

    pub const fn caller_material(&self) -> PhysicalMutationIdempotencyMaterial {
        self.material
    }
}
