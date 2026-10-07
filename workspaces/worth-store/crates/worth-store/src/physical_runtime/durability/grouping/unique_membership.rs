use sha2::{Digest, Sha256};
use worth_proof::{NonEmpty, UniqueVec};

use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalDurabilityGroupMemberBinding,
    PhysicalGroupAppendAmplificationObservation, PhysicalMutationIdempotencyKeyIdentity,
    PhysicalMutationIdentity, PhysicalWalMemberIdentity, PreparedPhysicalMutation,
    WalAppendedPhysicalMutation,
};

const MEMBERSHIP_DOMAIN: &[u8] = b"store.physical.durability-group-membership.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PhysicalGroupMembershipDigest([u8; 32]);

pub(super) struct PreparedGroupMembershipProof {
    mutation_identities: UniqueVec<PhysicalMutationIdentity>,
    member_identities: UniqueVec<PhysicalWalMemberIdentity>,
    idempotency_identities: UniqueVec<PhysicalMutationIdempotencyKeyIdentity>,
    digest: PhysicalGroupMembershipDigest,
}

pub struct WalBarrierMember<M> {
    binding: PhysicalDurabilityGroupMemberBinding,
    mutation: M,
}

pub struct SealedPhysicalDurabilityGroupMembers {
    basis: PhysicalDurabilityGroupBasis,
    members: NonEmpty<WalBarrierMember<WalAppendedPhysicalMutation>>,
    mutation_identities: UniqueVec<PhysicalMutationIdentity>,
    member_identities: UniqueVec<PhysicalWalMemberIdentity>,
    idempotency_identities: UniqueVec<PhysicalMutationIdempotencyKeyIdentity>,
}

pub(in crate::physical_runtime) struct PhysicalDurabilityGroupSealingFailure {
    members: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
    cause: PhysicalDurabilityGroupSealingDenial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDurabilityGroupSealingDenial {
    EmptyMembership,
    GroupIdentityMismatch,
    MemberCountMismatch,
    MembershipDigestMismatch,
    MemberIdentityMismatch,
    DuplicateMutationIdentity,
    DuplicateMemberIdentity,
    DuplicateIdempotencyIdentity,
    OverlappingWalRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedGroupMembershipDenial {
    Mutation,
    Member,
    Idempotency,
}

pub(super) fn prove_prepared_group_membership(
    members: &NonEmpty<PreparedPhysicalMutation>,
) -> Result<PreparedGroupMembershipProof, PreparedGroupMembershipDenial> {
    let mutation_identities = members
        .as_slice()
        .iter()
        .map(PreparedPhysicalMutation::mutation_identity)
        .collect::<Vec<_>>();
    let member_identities = mutation_identities
        .iter()
        .copied()
        .map(PhysicalWalMemberIdentity::for_mutation)
        .collect::<Vec<_>>();
    let idempotency_identities = members
        .as_slice()
        .iter()
        .map(PreparedPhysicalMutation::idempotency_identity)
        .collect::<Vec<_>>();
    let mutation_identities = UniqueVec::try_from_unique_preserving_order(mutation_identities)
        .map_err(|_| PreparedGroupMembershipDenial::Mutation)?;
    let member_identities = UniqueVec::try_from_unique_preserving_order(member_identities)
        .map_err(|_| PreparedGroupMembershipDenial::Member)?;
    let idempotency_identities =
        UniqueVec::try_from_unique_preserving_order(idempotency_identities)
            .map_err(|_| PreparedGroupMembershipDenial::Idempotency)?;
    let digest = membership_digest(
        mutation_identities.as_slice(),
        member_identities.as_slice(),
        idempotency_identities.as_slice(),
    );
    Ok(PreparedGroupMembershipProof {
        mutation_identities,
        member_identities,
        idempotency_identities,
        digest,
    })
}

impl PreparedGroupMembershipProof {
    pub(super) const fn digest(&self) -> PhysicalGroupMembershipDigest {
        self.digest
    }

    pub(super) fn mutation_identities(&self) -> &[PhysicalMutationIdentity] {
        self.mutation_identities.as_slice()
    }

    pub(super) fn member_identities(&self) -> &[PhysicalWalMemberIdentity] {
        self.member_identities.as_slice()
    }

    pub(super) fn idempotency_identities(&self) -> &[PhysicalMutationIdempotencyKeyIdentity] {
        self.idempotency_identities.as_slice()
    }
}

impl PhysicalGroupMembershipDigest {
    pub(in crate::physical_runtime) const fn from_reopened(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(super) const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

fn membership_digest(
    mutations: &[PhysicalMutationIdentity],
    members: &[PhysicalWalMemberIdentity],
    idempotency: &[PhysicalMutationIdempotencyKeyIdentity],
) -> PhysicalGroupMembershipDigest {
    let facts =
        mutations
            .iter()
            .zip(members)
            .zip(idempotency)
            .map(|((mutation, member), idempotency)| {
                (
                    mutation.store_identity().bytes(),
                    mutation.runtime_identity().get(),
                    mutation.operation_identity().get(),
                    member.bytes(),
                    idempotency.bytes(),
                )
            });
    PhysicalGroupMembershipDigest(
        reopened_membership_digest_fields(mutations.len(), facts)
            .expect("validated group membership has matching identity dimensions"),
    )
}

pub(in crate::physical_runtime) fn reopened_membership_digest_fields(
    count: usize,
    facts: impl IntoIterator<Item = ([u8; 16], u64, u64, [u8; 32], [u8; 32])>,
) -> Option<[u8; 32]> {
    let mut digest = Sha256::new();
    digest.update(MEMBERSHIP_DOMAIN);
    digest.update((count as u64).to_le_bytes());
    let mut seen = 0_usize;
    for (store, runtime, operation, member, idempotency) in facts {
        seen = seen.checked_add(1)?;
        digest.update(store);
        digest.update(runtime.to_le_bytes());
        digest.update(operation.to_le_bytes());
        digest.update(member);
        digest.update(idempotency);
    }
    (seen == count).then(|| digest.finalize().into())
}

pub(in crate::physical_runtime) fn reopened_membership_digest(
    mutations: &[PhysicalMutationIdentity],
    members: &[PhysicalWalMemberIdentity],
    idempotency: &[PhysicalMutationIdempotencyKeyIdentity],
) -> [u8; 32] {
    membership_digest(mutations, members, idempotency).bytes()
}

impl<M> WalBarrierMember<M> {
    pub(in crate::physical_runtime) fn new(
        binding: PhysicalDurabilityGroupMemberBinding,
        mutation: M,
    ) -> Self {
        Self { binding, mutation }
    }

    pub const fn binding(&self) -> PhysicalDurabilityGroupMemberBinding {
        self.binding
    }

    pub const fn mutation(&self) -> &M {
        &self.mutation
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (PhysicalDurabilityGroupMemberBinding, M) {
        (self.binding, self.mutation)
    }
}

impl SealedPhysicalDurabilityGroupMembers {
    pub(in crate::physical_runtime) fn seal(
        basis: PhysicalDurabilityGroupBasis,
        members: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
    ) -> Result<Self, PhysicalDurabilityGroupSealingFailure> {
        let validation = validate_sealing_members(basis, &members);
        let (mutation_identities, member_identities, idempotency_identities) = match validation {
            Ok(validation) => validation,
            Err(cause) => return Err(PhysicalDurabilityGroupSealingFailure { members, cause }),
        };
        let members = match NonEmpty::try_from_vec(members) {
            Ok(members) => members,
            Err(members) => {
                return Err(PhysicalDurabilityGroupSealingFailure {
                    members,
                    cause: PhysicalDurabilityGroupSealingDenial::EmptyMembership,
                })
            }
        };
        Ok(Self {
            basis,
            members,
            mutation_identities,
            member_identities,
            idempotency_identities,
        })
    }

    pub const fn basis(&self) -> PhysicalDurabilityGroupBasis {
        self.basis
    }

    pub fn members(&self) -> &[WalBarrierMember<WalAppendedPhysicalMutation>] {
        self.members.as_slice()
    }

    pub fn mutation_identities(&self) -> &[PhysicalMutationIdentity] {
        self.mutation_identities.as_slice()
    }

    pub fn member_identities(&self) -> &[PhysicalWalMemberIdentity] {
        self.member_identities.as_slice()
    }

    pub fn idempotency_identities(&self) -> &[PhysicalMutationIdempotencyKeyIdentity] {
        self.idempotency_identities.as_slice()
    }

    pub fn amplification_observation(&self) -> PhysicalGroupAppendAmplificationObservation {
        PhysicalGroupAppendAmplificationObservation::for_appended_group(self)
    }

    pub fn into_members(self) -> NonEmpty<WalBarrierMember<WalAppendedPhysicalMutation>> {
        self.members
    }
}

type SealingIdentityProof = (
    UniqueVec<PhysicalMutationIdentity>,
    UniqueVec<PhysicalWalMemberIdentity>,
    UniqueVec<PhysicalMutationIdempotencyKeyIdentity>,
);

fn validate_sealing_members(
    basis: PhysicalDurabilityGroupBasis,
    members: &[WalBarrierMember<WalAppendedPhysicalMutation>],
) -> Result<SealingIdentityProof, PhysicalDurabilityGroupSealingDenial> {
    if members.is_empty() {
        return Err(PhysicalDurabilityGroupSealingDenial::EmptyMembership);
    }
    if usize::try_from(basis.member_count().get()).ok() != Some(members.len()) {
        return Err(PhysicalDurabilityGroupSealingDenial::MemberCountMismatch);
    }
    let mutation_identities = members
        .iter()
        .map(|member| member.mutation().mutation_identity())
        .collect::<Vec<_>>();
    let member_identities = members
        .iter()
        .map(|member| {
            member
                .mutation()
                .reserved()
                .member_basis()
                .member_identity()
        })
        .collect::<Vec<_>>();
    let idempotency_identities = members
        .iter()
        .map(|member| member.mutation().reserved().idempotency_identity())
        .collect::<Vec<_>>();
    let expected_membership = membership_digest(
        &mutation_identities,
        &member_identities,
        &idempotency_identities,
    )
    .bytes();
    for (member, member_identity) in members.iter().zip(&member_identities) {
        let binding = member.binding();
        if binding.group_identity() != basis.identity() {
            return Err(PhysicalDurabilityGroupSealingDenial::GroupIdentityMismatch);
        }
        if binding.member_count() != basis.member_count() {
            return Err(PhysicalDurabilityGroupSealingDenial::MemberCountMismatch);
        }
        if binding.membership_digest() != expected_membership {
            return Err(PhysicalDurabilityGroupSealingDenial::MembershipDigestMismatch);
        }
        if binding.member_identity() != *member_identity {
            return Err(PhysicalDurabilityGroupSealingDenial::MemberIdentityMismatch);
        }
    }
    require_disjoint_wal_ranges(members)?;
    let mutation_identities = UniqueVec::try_from_unique_preserving_order(mutation_identities)
        .map_err(|_| PhysicalDurabilityGroupSealingDenial::DuplicateMutationIdentity)?;
    let member_identities = UniqueVec::try_from_unique_preserving_order(member_identities)
        .map_err(|_| PhysicalDurabilityGroupSealingDenial::DuplicateMemberIdentity)?;
    let idempotency_identities =
        UniqueVec::try_from_unique_preserving_order(idempotency_identities)
            .map_err(|_| PhysicalDurabilityGroupSealingDenial::DuplicateIdempotencyIdentity)?;
    Ok((
        mutation_identities,
        member_identities,
        idempotency_identities,
    ))
}

fn require_disjoint_wal_ranges(
    members: &[WalBarrierMember<WalAppendedPhysicalMutation>],
) -> Result<(), PhysicalDurabilityGroupSealingDenial> {
    let mut ranges = members
        .iter()
        .map(|member| {
            let range = member.mutation().reserved().member_basis().lsn_range();
            (range.start().get(), range.end_exclusive().get())
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(PhysicalDurabilityGroupSealingDenial::OverlappingWalRange);
    }
    Ok(())
}

impl PhysicalDurabilityGroupSealingFailure {
    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
        PhysicalDurabilityGroupSealingDenial,
    ) {
        (self.members, self.cause)
    }
}
