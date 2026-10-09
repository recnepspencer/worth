use std::num::NonZeroU32;

use super::super::super::test_support::{fingerprint, fixture, mutation, RegistryFixture};
use super::super::*;
use crate::physical_runtime::{
    PhysicalDurabilityGroupIdentity, PhysicalDurabilityGroupMemberBinding,
    PhysicalMutationIdempotencyMaterial, PhysicalWalMemberBasis, PhysicalWalMemberIdentity,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

/// Codec fields derive their Store, key, policy, and mutation from a real owner.
/// The local reopened group and LSN values test serialization, not WAL authority.
pub(in crate::physical_runtime::durability::mutation::idempotency) fn binding(
) -> (RegistryFixture, PersistedPhysicalMutationAttemptBinding) {
    let fixture = fixture(8);
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([109; 32]))
        .unwrap();
    let mutation = mutation(&fixture, 31);
    let member_identity = PhysicalWalMemberIdentity::for_mutation(mutation);
    let group = PhysicalDurabilityGroupMemberBinding::from_reopened(
        PhysicalDurabilityGroupIdentity::from_reopened([113; 32]),
        member_identity,
        NonZeroU32::new(1).unwrap(),
        NonZeroU32::new(1).unwrap(),
        crate::physical_runtime::durability::reopened_membership_digest(
            &[mutation],
            &[member_identity],
            &[key.identity()],
        ),
    )
    .unwrap();
    let range = WalLsnRange::new(LogSequenceNumber::new(3), LogSequenceNumber::new(4)).unwrap();
    let mut binding = PersistedPhysicalMutationAttemptBinding {
        key,
        fingerprint: fingerprint(&fixture, 17),
        mutation,
        group,
        member: PhysicalWalMemberBasis::new(member_identity, mutation, range),
        redo_digest: [127; 32],
        bytes: Box::default(),
    };
    binding.bytes = binding.encode().into_boxed_slice();
    (fixture, binding)
}
