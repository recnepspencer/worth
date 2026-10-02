//! Group-wide checks after arrival-ordered binding interpretation has settled.

use super::super::StoreRecoveryBindingSampleDenial as Denial;
use crate::physical_runtime::{
    PhysicalDurabilityGroupMemberBinding, PhysicalMutationIdempotencyKeyIdentity,
    PhysicalMutationIdentity,
};

pub(super) type RecoveryWalGroupBinding = (
    PhysicalMutationIdentity,
    PhysicalDurabilityGroupMemberBinding,
    PhysicalMutationIdempotencyKeyIdentity,
);

/// The caller has already funded the tuple roster and scratch capacity. Sorting
/// only this workspace leaves retained WAL members in their arrival order.
pub(super) fn validate_wal_groups(
    groups: &mut [RecoveryWalGroupBinding],
    scratch: &mut Vec<usize>,
) -> Result<(), Denial> {
    if scratch.capacity() < groups.len() {
        return Err(Denial::RecoveryMemoryLimit);
    }
    scratch.clear();
    groups.sort_unstable_by_key(|binding| {
        (
            binding.1.group_identity().bytes(),
            binding.1.ordinal().get(),
        )
    });
    let mut position = 0;
    while position < groups.len() {
        let start = position;
        let identity = groups[start].1.group_identity();
        while position < groups.len() && groups[position].1.group_identity() == identity {
            position += 1;
        }
        let group = &groups[start..position];
        validate_group(group.len(), |index| observed_member(&group[index]), scratch)?;
    }
    Ok(())
}

// Inline observations contain comparison/hash inputs, never submission or
// durability authority. This one validator is also the local semantic test seam.
#[derive(Clone, Copy)]
struct ObservedGroupMember {
    store: [u8; 16],
    runtime: u64,
    operation: u64,
    member: [u8; 32],
    idempotency: [u8; 32],
    ordinal: u32,
    count: u32,
    membership: [u8; 32],
}

fn observed_member(binding: &RecoveryWalGroupBinding) -> ObservedGroupMember {
    ObservedGroupMember {
        store: binding.0.store_identity().bytes(),
        runtime: binding.0.runtime_identity().get(),
        operation: binding.0.operation_identity().get(),
        member: binding.1.member_identity().bytes(),
        idempotency: binding.2.bytes(),
        ordinal: binding.1.ordinal().get(),
        count: binding.1.member_count().get(),
        membership: binding.1.membership_digest(),
    }
}

fn validate_group(
    len: usize,
    member_at: impl Fn(usize) -> ObservedGroupMember,
    scratch: &mut Vec<usize>,
) -> Result<(), Denial> {
    if len == 0 {
        return Err(Denial::InvalidWalMember);
    }
    if scratch.capacity() < len {
        return Err(Denial::RecoveryMemoryLimit);
    }
    let first = member_at(0);
    if len != first.count as usize {
        return Err(Denial::InvalidWalMember);
    }
    for index in 0..len {
        let member = member_at(index);
        if member.ordinal as usize != index + 1
            || member.count != first.count
            || member.membership != first.membership
        {
            return Err(Denial::InvalidWalMember);
        }
    }
    scratch.clear();
    // Capacity was admitted above; these pushes cannot grow the workspace.
    for index in 0..len {
        scratch.push(index);
    }
    scratch.sort_unstable_by_key(|index| member_at(*index).member);
    if scratch
        .windows(2)
        .any(|pair| member_at(pair[0]).member == member_at(pair[1]).member)
    {
        return Err(Denial::InvalidWalMember);
    }
    scratch.sort_unstable_by_key(|index| member_at(*index).idempotency);
    if scratch
        .windows(2)
        .any(|pair| member_at(pair[0]).idempotency == member_at(pair[1]).idempotency)
    {
        return Err(Denial::InvalidWalMember);
    }
    let digest = crate::physical_runtime::durability::reopened_membership_digest_fields(
        len,
        (0..len).map(|index| {
            let member = member_at(index);
            (
                member.store,
                member.runtime,
                member.operation,
                member.member,
                member.idempotency,
            )
        }),
    );
    if digest != Some(first.membership) {
        return Err(Denial::InvalidWalMember);
    }
    scratch.clear();
    Ok(())
}

#[cfg(test)]
mod tests;
