use sha2::{Digest, Sha256};
use worth_store_physical_backend::PhysicalDurabilityAdmissionIdentity;

use super::{
    CheckpointConfigured, GroupConfigured, GroupPolicy, IdempotencyConfigured,
    PhysicalCheckpointPolicy, PhysicalDurabilityDeclarationBuilder,
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalWalPolicy, WalConfigured,
};

const POLICY_IDENTITY_DOMAIN: &[u8] = b"worth.store.physical.durability.policy.v1";
const DECLARATION_IDENTITY_DOMAIN: &[u8] = b"worth.store.physical.durability.declaration.v1";

/// A fully declared durability policy not yet admitted over media.
///
/// Recovery admits it over the recovered media before the first checkpoint,
/// when no checkpoint carries the policy the WAL was written under.
pub type ConfiguredPhysicalDurabilityDeclaration = PhysicalDurabilityDeclarationBuilder<
    GroupConfigured,
    WalConfigured,
    IdempotencyConfigured,
    CheckpointConfigured,
>;

impl ConfiguredPhysicalDurabilityDeclaration {
    /// Digest of the declaration alone, without any media admission basis.
    pub fn declaration_identity(&self) -> [u8; 32] {
        let mut digest = domain(DECLARATION_IDENTITY_DOMAIN);
        declare(
            &mut digest,
            self.group.0,
            self.wal.0,
            self.idempotency.0,
            self.checkpoint.0,
        );
        digest.finalize().into()
    }
}

pub(super) fn policy_identity(
    basis: PhysicalDurabilityAdmissionIdentity,
    group: GroupPolicy,
    wal: PhysicalWalPolicy,
    idempotency: PhysicalIdempotencyPolicy,
    checkpoint: PhysicalCheckpointPolicy,
) -> PhysicalDurabilityPolicyIdentity {
    let mut digest = domain(POLICY_IDENTITY_DOMAIN);
    digest.update(basis.bytes());
    declare(&mut digest, group, wal, idempotency, checkpoint);
    PhysicalDurabilityPolicyIdentity(digest.finalize().into())
}

fn domain(domain: &[u8]) -> Sha256 {
    let mut digest = Sha256::new();
    digest.update((domain.len() as u64).to_le_bytes());
    digest.update(domain);
    digest
}

fn declare(
    digest: &mut Sha256,
    group: GroupPolicy,
    wal: PhysicalWalPolicy,
    idempotency: PhysicalIdempotencyPolicy,
    checkpoint: PhysicalCheckpointPolicy,
) {
    digest.update(group.limit.get().get().to_le_bytes());
    digest.update(group.delay.signal_duration().get().to_le_bytes());
    digest.update(wal.segment_byte_limit().get().get().to_le_bytes());
    digest.update(wal.segment_inventory_limit().get().get().to_le_bytes());
    digest.update(idempotency.retention.get().get().to_le_bytes());
    digest.update(idempotency.pending_unresolved.get().get().to_le_bytes());
    digest.update(idempotency.live_bindings.get().get().to_le_bytes());
    digest.update(checkpoint.memory.get().get().to_le_bytes());
    digest.update(checkpoint.retained_wal_tail.get().get().to_le_bytes());
}
