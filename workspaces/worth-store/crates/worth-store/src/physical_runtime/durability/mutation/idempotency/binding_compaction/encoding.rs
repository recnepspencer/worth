use super::super::canonical_encoding::CanonicalBindingEncoding;
use super::super::registry::PhysicalMutationBindingBasis;

pub(super) const COMPACTION_RECORD_DOMAIN: &[u8] =
    worth_store_physical_format::PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN;
pub(super) const STATE_UNSEALED: u8 = 1;
pub(super) const STATE_GROUP_SEALED: u8 = 2;
pub(super) const STATE_TERMINAL: u8 = 3;
pub(super) const STATE_WAL_BOUND: u8 = 4;

pub(super) fn encode_unsealed(basis: &PhysicalMutationBindingBasis) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(256);
    encode_unsealed_into(basis, &mut encoded);
    encoded
}

pub(super) fn encode_unsealed_into(
    basis: &PhysicalMutationBindingBasis,
    target: &mut impl CanonicalBindingEncoding,
) {
    encode_basis(STATE_UNSEALED, basis, target);
}

pub(super) fn encode_group_sealed(
    basis: &PhysicalMutationBindingBasis,
    group: crate::physical_runtime::PhysicalDurabilityGroupMemberBinding,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(256);
    encode_group_sealed_into(basis, group, &mut encoded);
    encoded
}

pub(super) fn encode_group_sealed_into(
    basis: &PhysicalMutationBindingBasis,
    group: crate::physical_runtime::PhysicalDurabilityGroupMemberBinding,
    target: &mut impl CanonicalBindingEncoding,
) {
    encode_basis(STATE_GROUP_SEALED, basis, target);
    target.field(&group.group_identity().bytes());
    target.field(&group.member_identity().bytes());
    target.write(&group.ordinal().get().to_le_bytes());
    target.write(&group.member_count().get().to_le_bytes());
    target.field(&group.membership_digest());
}

pub(super) fn encode_terminal(
    basis: &PhysicalMutationBindingBasis,
    fate: &super::super::fate::PersistedPhysicalMutationFate,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(256);
    encode_terminal_into(basis, fate, &mut encoded);
    encoded
}

pub(super) fn encode_terminal_into(
    basis: &PhysicalMutationBindingBasis,
    fate: &super::super::fate::PersistedPhysicalMutationFate,
    target: &mut impl CanonicalBindingEncoding,
) {
    encode_basis(STATE_TERMINAL, basis, target);
    fate.encode(target);
}

pub(super) fn encode_wal_bound(
    persisted: &super::super::PersistedPhysicalMutationAttemptBinding,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(persisted.bytes().len() + 80);
    encode_wal_bound_into(persisted, &mut encoded);
    encoded
}

pub(super) fn encode_wal_bound_into(
    persisted: &super::super::PersistedPhysicalMutationAttemptBinding,
    target: &mut impl CanonicalBindingEncoding,
) {
    target.field(COMPACTION_RECORD_DOMAIN);
    target.push(STATE_WAL_BOUND);
    target.field(persisted.bytes());
}

fn encode_basis(
    state: u8,
    basis: &PhysicalMutationBindingBasis,
    target: &mut impl CanonicalBindingEncoding,
) {
    let key = basis.key();
    let lease = key.lease();
    let mutation = basis.mutation();
    target.field(COMPACTION_RECORD_DOMAIN);
    target.push(state);
    target.field(&key.identity().bytes());
    target.field(&lease.store_identity().bytes());
    target.field(&lease.policy_identity().bytes());
    target.write(&lease.issuance_generation().get().to_le_bytes());
    target.write(&lease.expiry_generation().get().to_le_bytes());
    target.field(&key.caller_material().bytes());
    target.field(&basis.fingerprint().bytes());
    target.field(&mutation.store_identity().bytes());
    target.write(&mutation.runtime_identity().get().to_le_bytes());
    target.write(&mutation.lifecycle_generation().to_le_bytes());
    target.write(&mutation.operation_identity().get().to_le_bytes());
}
