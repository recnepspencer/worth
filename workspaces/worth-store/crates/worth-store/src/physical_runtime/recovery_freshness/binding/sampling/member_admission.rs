//! A decoded persisted owner cannot escape its distinct native scratch grant.

use super::super::{
    evidence::evidence_from_persisted, StoreRecoveryOperationEvidence, StoreRecoveryOperationFate,
    StoreRecoveryWalMember,
};
use super::{
    allocation::{SamplingAllocation, StoreRecoveryBindingSampleAllocationDenial},
    group_validation::RecoveryWalGroupBinding,
};
use crate::physical_runtime::durability::{
    PersistedPhysicalMutationAttemptBinding, PhysicalBindingDecodingContext,
    PhysicalPersistedBindingDecodeDenial,
};
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;
use sha2::{Digest, Sha256};
use worth_store_wal::WalLsnRange;

pub(super) enum MemberDenial {
    Semantic,
    Allocation(StoreRecoveryBindingSampleAllocationDenial),
}

pub(super) fn admit(
    allocation: &SamplingAllocation<'_>,
    bytes: &[u8],
    redo: &[u8],
    context: PhysicalBindingDecodingContext,
    range: WalLsnRange,
    generation: u64,
) -> Result<
    (
        StoreRecoveryOperationEvidence,
        RecoveryWalGroupBinding,
        StoreRecoveryWalMember,
    ),
    MemberDenial,
> {
    use StoreRecoveryBindingSampleAllocationDenial as Denial;
    let requested = (bytes.len() as u64)
        .checked_mul(2)
        .ok_or(MemberDenial::Allocation(Denial::SizeOverflow))?;
    let _backing = allocation
        .reserve(requested)
        .map_err(MemberDenial::Allocation)?;
    let binding = PersistedPhysicalMutationAttemptBinding::decode_from_wal_member(
        bytes,
        context,
        range,
        Sha256::digest(redo).into(),
    )
    .map_err(|denial| match denial {
        PhysicalPersistedBindingDecodeDenial::Allocation { requested, cause } => {
            MemberDenial::Allocation(Denial::Backing {
                requested,
                cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
            })
        }
        PhysicalPersistedBindingDecodeDenial::AllocatorExceededReservation {
            requested,
            actual,
        } => MemberDenial::Allocation(Denial::AllocatorExceededReservation { requested, actual }),
        _ => MemberDenial::Semantic,
    })?;
    let group = binding.group();
    let member = StoreRecoveryWalMember {
        lsn_range: range,
        operation_identity: binding.idempotency_identity().bytes(),
        group_identity: group.group_identity().bytes(),
        group_member_identity: group.member_identity().bytes(),
        group_member_ordinal: group.ordinal().get(),
        group_member_count: group.member_count().get(),
        group_membership_digest: group.membership_digest(),
        canonical_redo: Vec::new(),
    };
    Ok((
        evidence_from_persisted(
            &binding,
            generation,
            StoreRecoveryOperationFate::Indeterminate,
        ),
        (binding.mutation(), group, binding.idempotency_identity()),
        member,
    ))
}
