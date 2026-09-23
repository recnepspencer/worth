use std::num::{NonZeroU32, NonZeroU64};
use std::sync::{Arc, Mutex, Weak};

use worth_proof::NonEmpty;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

mod identity_issuance;

use super::admission_validation::{validate_members, PhysicalGroupAdmissionContext};
use super::root_publication::PhysicalGroupRootPublicationPlan;
use super::unique_membership::PhysicalGroupMembershipDigest;
use crate::physical_runtime::durability::{
    PhysicalMutationGroupSealingBinding, PhysicalMutationUnresolvedBindingObservation,
};
use crate::physical_runtime::{
    PhysicalDurabilityObservation, PhysicalDurabilityPolicyIdentity,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity, PhysicalWalMemberIdentity,
    PreparedPhysicalMutation, RuntimeIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalDurabilityGroupIdentity([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhysicalGroupMemberOrdinal(NonZeroU32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalGroupQueueAdmissionTick(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalDurabilityGroupMemberBinding {
    group: PhysicalDurabilityGroupIdentity,
    member: PhysicalWalMemberIdentity,
    ordinal: PhysicalGroupMemberOrdinal,
    member_count: NonZeroU32,
    membership: PhysicalGroupMembershipDigest,
}

pub struct AdmittedPhysicalDurabilityGroupMember {
    pub(in crate::physical_runtime) prepared: PreparedPhysicalMutation,
    binding: PhysicalDurabilityGroupMemberBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalDurabilityGroupBasis {
    identity: PhysicalDurabilityGroupIdentity,
    membership: PhysicalGroupMembershipDigest,
    root_plan: PhysicalGroupRootPublicationPlan,
    aggregate_bytes: u64,
    oldest_queue_age: u64,
    member_count: NonZeroU32,
}

pub struct AdmittedPhysicalDurabilityGroup {
    basis: PhysicalDurabilityGroupBasis,
    members: NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
}

pub struct RejectedPhysicalDurabilityGroup {
    members: NonEmpty<PreparedPhysicalMutation>,
    cause: PhysicalDurabilityGroupAdmissionDenial,
}

pub type PhysicalDurabilityGroupAdmissionOutcome =
    Result<AdmittedPhysicalDurabilityGroup, RejectedPhysicalDurabilityGroup>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDurabilityGroupAdmissionDenial {
    AuthorityReleased,
    GroupIdentityExhausted,
    WidthExceeded { admitted: u32, requested: u32 },
    AggregateBytesExceeded { admitted: u64, requested: u64 },
    QueueClockRegressed,
    QueueAgeExceeded { admitted: u64, observed: u64 },
    DuplicateMutationIdentity,
    DuplicateMemberIdentity,
    DuplicateIdempotencyIdentity,
    DuplicateUnresolvedMutation,
    ForeignStore,
    ForeignRuntime,
    DurabilityPolicyMismatch,
    SignalProfileMismatch,
    DurabilityBasisMismatch,
    IdempotencyAuthorityReleased,
    IdempotencyBindingMismatch,
    IdempotencyAlreadyGroupSealed,
    IdempotencyProvenNoEffect,
    IdempotencyReopenedUnresolved,
}

pub(in crate::physical_runtime) struct PhysicalDurabilityGroupingRuntimeOwner {
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    policy: PhysicalDurabilityPolicyIdentity,
    next_sequence: Mutex<NonZeroU64>,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalDurabilityGroupingRuntimeAuthority {
    owner: Weak<PhysicalDurabilityGroupingRuntimeOwner>,
}

impl PhysicalDurabilityGroupingRuntimeOwner {
    pub(in crate::physical_runtime) fn new(
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        policy: PhysicalDurabilityPolicyIdentity,
    ) -> Arc<Self> {
        Arc::new(Self {
            store,
            runtime,
            policy,
            next_sequence: Mutex::new(NonZeroU64::MIN),
        })
    }

    pub(in crate::physical_runtime) fn authority(
        owner: &Arc<Self>,
    ) -> PhysicalDurabilityGroupingRuntimeAuthority {
        PhysicalDurabilityGroupingRuntimeAuthority {
            owner: Arc::downgrade(owner),
        }
    }
}

impl PhysicalDurabilityGroupingRuntimeAuthority {
    pub(in crate::physical_runtime) fn admit(
        &self,
        members: NonEmpty<PreparedPhysicalMutation>,
        durability: PhysicalDurabilityObservation,
        current_tick: u64,
        aggregate_byte_limit: u64,
    ) -> PhysicalDurabilityGroupAdmissionOutcome {
        let Some(owner) = self.owner.upgrade() else {
            return Err(rejected(
                members,
                PhysicalDurabilityGroupAdmissionDenial::AuthorityReleased,
            ));
        };
        let context = PhysicalGroupAdmissionContext::new(
            owner.store,
            owner.runtime,
            owner.policy,
            durability,
            current_tick,
            aggregate_byte_limit,
        );
        let proof = match validate_members(&members, context) {
            Ok(proof) => proof,
            Err(cause) => return Err(rejected(members, cause)),
        };
        let identity = match owner.issue_identity(proof.membership.digest()) {
            Ok(identity) => identity,
            Err(cause) => return Err(rejected(members, cause)),
        };
        let member_count =
            NonZeroU32::new(members.len() as u32).expect("NonEmpty group membership is nonzero");
        let admitted = members
            .into_vec()
            .into_iter()
            .zip(proof.membership.member_identities().iter().copied())
            .enumerate()
            .map(|(index, (prepared, member))| {
                let ordinal = PhysicalGroupMemberOrdinal(
                    NonZeroU32::new(index as u32 + 1)
                        .expect("one-based group member ordinals are nonzero"),
                );
                AdmittedPhysicalDurabilityGroupMember {
                    prepared,
                    binding: PhysicalDurabilityGroupMemberBinding {
                        group: identity,
                        member,
                        ordinal,
                        member_count,
                        membership: proof.membership.digest(),
                    },
                }
            })
            .collect::<Vec<_>>();
        let admitted = match NonEmpty::try_from_vec(admitted) {
            Ok(admitted) => admitted,
            Err(_) => unreachable!("admission preserves nonempty membership"),
        };
        Ok(AdmittedPhysicalDurabilityGroup {
            basis: PhysicalDurabilityGroupBasis {
                identity,
                membership: proof.membership.digest(),
                root_plan: PhysicalGroupRootPublicationPlan::for_group(
                    identity,
                    proof.membership.digest(),
                    admitted.len(),
                ),
                aggregate_bytes: proof.aggregate_bytes,
                oldest_queue_age: proof.oldest_queue_age,
                member_count,
            },
            members: admitted,
        })
    }
}

fn rejected(
    members: NonEmpty<PreparedPhysicalMutation>,
    cause: PhysicalDurabilityGroupAdmissionDenial,
) -> RejectedPhysicalDurabilityGroup {
    RejectedPhysicalDurabilityGroup { members, cause }
}

impl PhysicalDurabilityGroupIdentity {
    pub(in crate::physical_runtime) const fn from_reopened(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl PhysicalGroupMemberOrdinal {
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl PhysicalGroupQueueAdmissionTick {
    pub(in crate::physical_runtime) const fn new(tick: u64) -> Self {
        Self(tick)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl PhysicalDurabilityGroupMemberBinding {
    pub(in crate::physical_runtime) fn from_reopened(
        group: PhysicalDurabilityGroupIdentity,
        member: PhysicalWalMemberIdentity,
        ordinal: NonZeroU32,
        member_count: NonZeroU32,
        membership: [u8; 32],
    ) -> Option<Self> {
        if ordinal.get() > member_count.get() {
            return None;
        }
        Some(Self {
            group,
            member,
            ordinal: PhysicalGroupMemberOrdinal(ordinal),
            member_count,
            membership: PhysicalGroupMembershipDigest::from_reopened(membership),
        })
    }

    pub const fn group_identity(self) -> PhysicalDurabilityGroupIdentity {
        self.group
    }

    pub const fn member_identity(self) -> PhysicalWalMemberIdentity {
        self.member
    }

    pub const fn ordinal(self) -> PhysicalGroupMemberOrdinal {
        self.ordinal
    }

    pub const fn member_count(self) -> NonZeroU32 {
        self.member_count
    }

    pub const fn membership_digest(self) -> [u8; 32] {
        self.membership.bytes()
    }
}

impl AdmittedPhysicalDurabilityGroupMember {
    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.prepared.mutation_identity()
    }

    pub const fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.prepared.idempotency_identity()
    }

    pub const fn binding(&self) -> PhysicalDurabilityGroupMemberBinding {
        self.binding
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PreparedPhysicalMutation,
        PhysicalDurabilityGroupMemberBinding,
    ) {
        (self.prepared, self.binding)
    }

    pub(in crate::physical_runtime) fn from_parts(
        prepared: PreparedPhysicalMutation,
        binding: PhysicalDurabilityGroupMemberBinding,
    ) -> Self {
        Self { prepared, binding }
    }
}

impl AdmittedPhysicalDurabilityGroup {
    pub const fn identity(&self) -> PhysicalDurabilityGroupIdentity {
        self.basis.identity
    }

    pub const fn root_publication_plan(&self) -> PhysicalGroupRootPublicationPlan {
        self.basis.root_plan
    }

    pub const fn aggregate_bytes(&self) -> u64 {
        self.basis.aggregate_bytes
    }

    pub const fn oldest_queue_age(&self) -> u64 {
        self.basis.oldest_queue_age
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub(in crate::physical_runtime) fn idempotency_sealing_bindings(
        &self,
    ) -> Vec<PhysicalMutationGroupSealingBinding> {
        self.members
            .as_slice()
            .iter()
            .map(|member| {
                PhysicalMutationGroupSealingBinding::new(
                    PhysicalMutationUnresolvedBindingObservation::new(
                        member.prepared.idempotency_identity(),
                        member.prepared.request_fingerprint(),
                        member.prepared.mutation_identity(),
                    ),
                    member.binding,
                )
            })
            .collect()
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PhysicalDurabilityGroupBasis,
        NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
    ) {
        (self.basis, self.members)
    }
}

impl PhysicalDurabilityGroupBasis {
    pub const fn identity(self) -> PhysicalDurabilityGroupIdentity {
        self.identity
    }

    pub const fn root_publication_plan(self) -> PhysicalGroupRootPublicationPlan {
        self.root_plan
    }

    pub const fn membership_digest(self) -> [u8; 32] {
        self.membership.bytes()
    }

    pub const fn aggregate_bytes(self) -> u64 {
        self.aggregate_bytes
    }

    pub const fn oldest_queue_age(self) -> u64 {
        self.oldest_queue_age
    }

    pub const fn member_count(self) -> NonZeroU32 {
        self.member_count
    }
}

impl RejectedPhysicalDurabilityGroup {
    pub(in crate::physical_runtime) fn before_effect(
        members: NonEmpty<PreparedPhysicalMutation>,
        cause: PhysicalDurabilityGroupAdmissionDenial,
    ) -> Self {
        rejected(members, cause)
    }

    pub const fn cause(&self) -> PhysicalDurabilityGroupAdmissionDenial {
        self.cause
    }

    pub fn into_members(self) -> NonEmpty<PreparedPhysicalMutation> {
        self.members
    }
}
