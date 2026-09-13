use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::durability::checkpoint::{
    PhysicalBindingCompactionReopenFailure, ReopenedPhysicalBindingCompaction,
};
use crate::physical_runtime::durability::{
    grouping::reopened_membership_digest, wal::inventory::ReopenedPhysicalWalMember,
};
use crate::physical_runtime::{
    PhysicalDurabilityGroupMemberBinding, PhysicalDurabilityPolicyIdentity,
    PhysicalIdempotencyPolicy, PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity,
    PhysicalWalMemberIdentity, RuntimeIdentity,
};

use super::binding_compaction::DecodedPhysicalMutationBindingRecord;
use super::persisted_binding::PhysicalBindingDecodingContext;
use super::registry::{
    PhysicalMutationIdempotencyBindingState, PhysicalMutationIdempotencyRegistry,
    PhysicalMutationWalBindingDenial, RebuiltPhysicalMutationBindingState,
};
use super::runtime_owner::PhysicalMutationIdempotencyRuntimeOwner;
use super::PhysicalNamespaceDurableCheckpointGeneration;

mod checkpoint_compaction;
#[cfg(test)]
#[path = "bootstrap/tests.rs"]
mod tests;

pub(in crate::physical_runtime) struct RebuiltPhysicalMutationIdempotency {
    owner: Arc<PhysicalMutationIdempotencyRuntimeOwner>,
    checkpoint:
        crate::physical_runtime::durability::checkpoint::PhysicalBindingCompactionReopenCounters,
    wal_members_read: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIdempotencyReopenFailure {
    Checkpoint(PhysicalBindingCompactionReopenFailure),
    CompactionRecordRejected,
    WalBindingRejected,
    NonCanonicalCompactionOrder,
    DuplicateBinding,
    LiveBindingLimitExceeded,
    PendingUnresolvedLimitExceeded,
    LeaseIssuedAfterDurableGeneration,
    WalTailDiscontinuity,
    WalBindingConflict,
    GroupMemberCountMismatch,
    GroupOrdinalMismatch,
    GroupMembershipMismatch,
    GroupIdentityCollision,
    CounterOverflow,
}

pub(in crate::physical_runtime) fn rebuild_idempotency(
    media: &QualifiedFilesystemMedia,
    runtime: RuntimeIdentity,
    policy: PhysicalDurabilityPolicyIdentity,
    idempotency: PhysicalIdempotencyPolicy,
    checkpoint: &ReopenedPhysicalBindingCompaction,
    wal_members: Vec<ReopenedPhysicalWalMember>,
) -> Result<RebuiltPhysicalMutationIdempotency, PhysicalIdempotencyReopenFailure> {
    let store = media.store_identity();
    let context = PhysicalBindingDecodingContext::new(store, policy, idempotency);
    let generation = checkpoint
        .wal_cutoff_lsn_exclusive()
        .map(|_| match checkpoint {
            ReopenedPhysicalBindingCompaction::NamespaceDurable(reopened) => {
                PhysicalNamespaceDurableCheckpointGeneration::from_reopened(reopened.generation())
            }
            ReopenedPhysicalBindingCompaction::GenerationZero => unreachable!(),
        })
        .unwrap_or(PhysicalNamespaceDurableCheckpointGeneration::INITIAL);
    let mut builder = PhysicalIdempotencyRegistryRebuilder::new(
        store,
        runtime,
        policy,
        idempotency,
        generation,
        context,
        checkpoint.wal_cutoff_lsn_exclusive(),
    );
    let (checkpoint_counters, checkpoint_admission) = match checkpoint {
        ReopenedPhysicalBindingCompaction::GenerationZero => (
            Default::default(),
            checkpoint_compaction::admit_generation_zero(),
        ),
        ReopenedPhysicalBindingCompaction::NamespaceDurable(reopened) => {
            checkpoint_compaction::rebuild(reopened, media, &mut builder)?
        }
    };
    let wal_members_read = u64::try_from(wal_members.len())
        .map_err(|_| PhysicalIdempotencyReopenFailure::CounterOverflow)?;
    for member in wal_members {
        builder.consume_wal_member(member)?;
    }
    builder.validate_groups()?;
    let registry = builder.finish(checkpoint_admission);
    Ok(RebuiltPhysicalMutationIdempotency {
        owner: PhysicalMutationIdempotencyRuntimeOwner::from_rebuilt_registry(registry),
        checkpoint: checkpoint_counters,
        wal_members_read,
    })
}

struct PhysicalIdempotencyRegistryRebuilder {
    registry: PhysicalMutationIdempotencyRegistry,
    context: PhysicalBindingDecodingContext,
    generation: PhysicalNamespaceDurableCheckpointGeneration,
    last_compaction_key: Option<PhysicalMutationIdempotencyKeyIdentity>,
    next_tail_lsn: Option<u64>,
    pending_count: usize,
    live_limit: usize,
    pending_limit: usize,
}

impl PhysicalIdempotencyRegistryRebuilder {
    fn new(
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        policy: PhysicalDurabilityPolicyIdentity,
        idempotency: PhysicalIdempotencyPolicy,
        generation: PhysicalNamespaceDurableCheckpointGeneration,
        context: PhysicalBindingDecodingContext,
        wal_cutoff_lsn_exclusive: Option<u64>,
    ) -> Self {
        let mut registry = PhysicalMutationIdempotencyRegistry::generation_zero(
            store,
            runtime,
            policy,
            idempotency,
        );
        registry.generation = generation;
        Self {
            registry,
            context,
            generation,
            last_compaction_key: None,
            next_tail_lsn: wal_cutoff_lsn_exclusive,
            pending_count: 0,
            live_limit: idempotency.live_binding_limit().get().get() as usize,
            pending_limit: idempotency.pending_unresolved_limit().get().get() as usize,
        }
    }

    fn consume_compaction_record(
        &mut self,
        record: &[u8],
    ) -> Result<(), PhysicalIdempotencyReopenFailure> {
        let decoded = DecodedPhysicalMutationBindingRecord::decode(record, self.context)
            .map_err(|_denial| PhysicalIdempotencyReopenFailure::CompactionRecordRejected)?;
        let (identity, issuance, state, unresolved): (
            PhysicalMutationIdempotencyKeyIdentity,
            PhysicalNamespaceDurableCheckpointGeneration,
            PhysicalMutationIdempotencyBindingState,
            bool,
        ) = match decoded {
            DecodedPhysicalMutationBindingRecord::RebuiltUnsealed(basis) => (
                basis.key().identity(),
                basis.key().lease().issuance_generation(),
                PhysicalMutationIdempotencyBindingState::RebuiltUnresolved {
                    basis,
                    prior: RebuiltPhysicalMutationBindingState::Unsealed,
                },
                true,
            ),
            DecodedPhysicalMutationBindingRecord::RebuiltGroupSealed { basis, group } => (
                basis.key().identity(),
                basis.key().lease().issuance_generation(),
                PhysicalMutationIdempotencyBindingState::RebuiltUnresolved {
                    basis,
                    prior: RebuiltPhysicalMutationBindingState::GroupSealed(group),
                },
                true,
            ),
            DecodedPhysicalMutationBindingRecord::Terminal { basis, fate } => (
                basis.key().identity(),
                basis.key().lease().issuance_generation(),
                PhysicalMutationIdempotencyBindingState::Terminal {
                    basis,
                    fate,
                    last_compacted: Some(self.generation),
                },
                false,
            ),
            DecodedPhysicalMutationBindingRecord::WalBound { basis, persisted } => (
                basis.key().identity(),
                basis.key().lease().issuance_generation(),
                PhysicalMutationIdempotencyBindingState::WalBound { basis, persisted },
                true,
            ),
        };
        if self
            .last_compaction_key
            .is_some_and(|previous| previous >= identity)
        {
            return Err(PhysicalIdempotencyReopenFailure::NonCanonicalCompactionOrder);
        }
        if issuance.get() >= self.generation.get() {
            return Err(PhysicalIdempotencyReopenFailure::LeaseIssuedAfterDurableGeneration);
        }
        self.insert(identity, state, unresolved)?;
        self.last_compaction_key = Some(identity);
        Ok(())
    }

    fn consume_wal_member(
        &mut self,
        member: ReopenedPhysicalWalMember,
    ) -> Result<(), PhysicalIdempotencyReopenFailure> {
        if let Some(next) = self.next_tail_lsn {
            if member.lsn_range().start().get() != next {
                return Err(PhysicalIdempotencyReopenFailure::WalTailDiscontinuity);
            }
        }
        self.next_tail_lsn = Some(member.lsn_range().end_exclusive().get());
        let persisted = super::PersistedPhysicalMutationAttemptBinding::decode_from_wal_member(
            member.persisted_binding(),
            self.context,
            member.lsn_range(),
            member.redo_digest(),
        )
        .map_err(|_denial| PhysicalIdempotencyReopenFailure::WalBindingRejected)?;
        if persisted.key().lease().issuance_generation().get() > self.generation.get() {
            return Err(PhysicalIdempotencyReopenFailure::LeaseIssuedAfterDurableGeneration);
        }
        let identity = persisted.key().identity();
        if self.registry.bindings.contains_key(&identity) {
            classify_reopened_wal_binding(self.registry.record_wal_binding(persisted))?;
            return Ok(());
        }
        let basis = super::registry::PhysicalMutationBindingBasis::new(
            persisted.key().clone(),
            persisted.fingerprint(),
            persisted.mutation(),
        );
        self.insert(
            identity,
            PhysicalMutationIdempotencyBindingState::WalBound { basis, persisted },
            true,
        )
    }

    fn insert(
        &mut self,
        identity: PhysicalMutationIdempotencyKeyIdentity,
        state: PhysicalMutationIdempotencyBindingState,
        unresolved: bool,
    ) -> Result<(), PhysicalIdempotencyReopenFailure> {
        if self.registry.bindings.len() >= self.live_limit {
            return Err(PhysicalIdempotencyReopenFailure::LiveBindingLimitExceeded);
        }
        if unresolved {
            self.pending_count = self
                .pending_count
                .checked_add(1)
                .ok_or(PhysicalIdempotencyReopenFailure::CounterOverflow)?;
            if self.pending_count > self.pending_limit {
                return Err(PhysicalIdempotencyReopenFailure::PendingUnresolvedLimitExceeded);
            }
        }
        if self.registry.bindings.insert(identity, state).is_some() {
            return Err(PhysicalIdempotencyReopenFailure::DuplicateBinding);
        }
        Ok(())
    }

    fn validate_groups(&self) -> Result<(), PhysicalIdempotencyReopenFailure> {
        let mut groups = BTreeMap::<[u8; 32], Vec<ReopenedGroupMember>>::new();
        for (key, state) in &self.registry.bindings {
            let member = match state {
                PhysicalMutationIdempotencyBindingState::GroupSealed { basis, group }
                | PhysicalMutationIdempotencyBindingState::RebuiltUnresolved {
                    basis,
                    prior: RebuiltPhysicalMutationBindingState::GroupSealed(group),
                } => Some(ReopenedGroupMember::new(*key, basis.mutation(), *group)),
                PhysicalMutationIdempotencyBindingState::WalBound { basis, persisted } => Some(
                    ReopenedGroupMember::new(*key, basis.mutation(), persisted.group()),
                ),
                _ => None,
            };
            if let Some(member) = member {
                groups
                    .entry(member.binding.group_identity().bytes())
                    .or_default()
                    .push(member);
            }
        }
        for members in groups.values_mut() {
            validate_group(members)?;
        }
        Ok(())
    }

    fn finish(
        self,
        _checkpoint_admission: checkpoint_compaction::CheckpointAggregateAdmission,
    ) -> PhysicalMutationIdempotencyRegistry {
        self.registry
    }
}

fn classify_reopened_wal_binding(
    result: Result<(), PhysicalMutationWalBindingDenial>,
) -> Result<(), PhysicalIdempotencyReopenFailure> {
    match result {
        Ok(()) | Err(PhysicalMutationWalBindingDenial::AlreadyWalBound) => Ok(()),
        Err(_) => Err(PhysicalIdempotencyReopenFailure::WalBindingConflict),
    }
}

struct ReopenedGroupMember {
    key: PhysicalMutationIdempotencyKeyIdentity,
    mutation: PhysicalMutationIdentity,
    member: PhysicalWalMemberIdentity,
    binding: PhysicalDurabilityGroupMemberBinding,
}

impl ReopenedGroupMember {
    fn new(
        key: PhysicalMutationIdempotencyKeyIdentity,
        mutation: PhysicalMutationIdentity,
        binding: PhysicalDurabilityGroupMemberBinding,
    ) -> Self {
        Self {
            key,
            mutation,
            member: binding.member_identity(),
            binding,
        }
    }
}

fn validate_group(
    members: &mut [ReopenedGroupMember],
) -> Result<(), PhysicalIdempotencyReopenFailure> {
    members.sort_unstable_by_key(|member| member.binding.ordinal().get());
    let expected_count = members[0].binding.member_count().get() as usize;
    if members.len() != expected_count {
        return Err(PhysicalIdempotencyReopenFailure::GroupMemberCountMismatch);
    }
    let expected_membership = members[0].binding.membership_digest();
    let mut keys = BTreeSet::new();
    let mut member_identities = BTreeSet::new();
    for (index, member) in members.iter().enumerate() {
        if member.binding.ordinal().get() as usize != index + 1 {
            return Err(PhysicalIdempotencyReopenFailure::GroupOrdinalMismatch);
        }
        if member.binding.member_count().get() as usize != expected_count
            || member.binding.membership_digest() != expected_membership
        {
            return Err(PhysicalIdempotencyReopenFailure::GroupIdentityCollision);
        }
        if !keys.insert(member.key.bytes()) || !member_identities.insert(member.member.bytes()) {
            return Err(PhysicalIdempotencyReopenFailure::GroupIdentityCollision);
        }
    }
    let mutations = members
        .iter()
        .map(|member| member.mutation)
        .collect::<Vec<_>>();
    let wal_members = members
        .iter()
        .map(|member| member.member)
        .collect::<Vec<_>>();
    let idempotency = members.iter().map(|member| member.key).collect::<Vec<_>>();
    if reopened_membership_digest(&mutations, &wal_members, &idempotency) != expected_membership {
        return Err(PhysicalIdempotencyReopenFailure::GroupMembershipMismatch);
    }
    Ok(())
}

impl RebuiltPhysicalMutationIdempotency {
    pub(in crate::physical_runtime) fn into_owner(
        self,
    ) -> Arc<PhysicalMutationIdempotencyRuntimeOwner> {
        self.owner
    }

    pub(in crate::physical_runtime) const fn checkpoint_counters(
        &self,
    ) -> crate::physical_runtime::durability::checkpoint::PhysicalBindingCompactionReopenCounters
    {
        self.checkpoint
    }

    pub(in crate::physical_runtime) const fn wal_members_read(&self) -> u64 {
        self.wal_members_read
    }
}
