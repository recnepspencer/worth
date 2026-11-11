use super::BridgeSemanticClaimOrigin;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::RwLockWriteGuard;

use crate::facade::RuntimeBridge;
use crate::mapping::SliceWideningPolicy;

use super::admission::CorrespondenceAdmissionOutcome;
use super::slot_allocation::{
    allocate_slot, allocation_target_identity, signal_node_admits, AllocationKey,
    AuthoritativeAllocationRecord, CorrespondenceAllocationRegistry, SlotAllocation,
};
use super::{
    BridgeCorrespondenceAdmissionFailure, BridgeCorrespondenceDeferred,
    BridgeCorrespondenceDenialKind, BridgeCorrespondencePrecision, InstalledCorrespondenceTarget,
    ProvenCorrespondenceTargets,
};

pub(super) struct AllocatedCorrespondence<'a> {
    pub(super) planned: PlannedCorrespondence,
    pub(super) registry: RwLockWriteGuard<'a, CorrespondenceAllocationRegistry>,
}

pub(super) struct PlannedCorrespondence {
    pub(super) resolved: super::resolution::ResolvedCorrespondence,
    pub(super) targets: ProvenCorrespondenceTargets,
    pub(super) pending_records: Vec<AuthoritativeAllocationRecord>,
}

pub(super) fn allocate_targets<'a>(
    runtime: &'a RuntimeBridge,
    mut mapped: super::target_mapping::MappedCorrespondence,
) -> Result<AllocatedCorrespondence<'a>, CorrespondenceAdmissionOutcome> {
    mapped.resolved.counters.allocation_registry_lock_attempts += 1;
    let registry = match runtime.correspondence_allocations.try_write() {
        Ok(registry) => registry,
        Err(std::sync::TryLockError::WouldBlock) => {
            return Err(worth_proof::TransitionOutcome::Deferred(
                BridgeCorrespondenceDeferred::GraphMutationInProgress,
            ))
        }
        Err(std::sync::TryLockError::Poisoned(_)) => {
            return Err(worth_proof::TransitionOutcome::Failed(
                BridgeCorrespondenceAdmissionFailure::LockPoisoned,
            ))
        }
    };
    allocate_with_registry(mapped, registry)
}

fn allocate_with_registry<'a>(
    mapped: super::target_mapping::MappedCorrespondence,
    registry: RwLockWriteGuard<'a, CorrespondenceAllocationRegistry>,
) -> Result<AllocatedCorrespondence<'a>, CorrespondenceAdmissionOutcome> {
    let mut pending_owners = BTreeMap::new();
    let planned = plan_with_registry(mapped, &registry, &mut pending_owners)?;
    Ok(AllocatedCorrespondence { planned, registry })
}

pub(super) fn plan_with_registry(
    mut mapped: super::target_mapping::MappedCorrespondence,
    registry: &CorrespondenceAllocationRegistry,
    pending_owners: &mut BTreeMap<AllocationKey, BTreeSet<BridgeSemanticClaimOrigin>>,
) -> Result<PlannedCorrespondence, CorrespondenceAdmissionOutcome> {
    let mut targets = Vec::with_capacity(mapped.targets.len());
    let mut pending_records = Vec::with_capacity(mapped.targets.len());
    for mapped_target in mapped.targets {
        let declaration = mapped_target.declaration;
        let target_identity = allocation_target_identity(&declaration);
        let (allocated, keys_examined) = allocate_slot(
            registry,
            pending_owners,
            mapped.resolved.signal_graph.graph_instance_id(),
            &declaration,
            &mapped.resolved.owner,
            &target_identity,
        );
        mapped.resolved.counters.allocation_keys_examined += keys_examined;
        let aspect = match allocated {
            SlotAllocation::Allocated(aspect) => aspect,
            SlotAllocation::CapacityExhausted => {
                mapped.resolved.counters.capacity_denials += 1;
                return Err(super::admission::denied(
                    BridgeCorrespondenceDenialKind::CapacityExhausted,
                    mapped.resolved.counters,
                ));
            }
            SlotAllocation::NodeContractMismatch => {
                return Err(super::admission::denied(
                    BridgeCorrespondenceDenialKind::SignalNodeContractMismatch,
                    mapped.resolved.counters,
                ));
            }
        };
        if !signal_node_admits(&declaration, aspect) {
            return Err(super::admission::denied(
                BridgeCorrespondenceDenialKind::SignalNodeContractMismatch,
                mapped.resolved.counters,
            ));
        }
        let key = AllocationKey {
            signal_graph_instance_id: mapped.resolved.signal_graph.graph_instance_id(),
            partition: declaration.partition.clone(),
            node: declaration.node,
            aspect,
        };
        mapped.resolved.counters.allocation_owner_lookups += 1;
        let existing_owners = registry.owners.get(&key);
        let pending_for_key = pending_owners.get(&key);
        if existing_owners.is_some() || pending_for_key.is_some() {
            let same_owner = existing_owners
                .is_some_and(|owners| owners.contains(&mapped.resolved.owner))
                || pending_for_key.is_some_and(|owners| owners.contains(&mapped.resolved.owner));
            if !same_owner && mapped_target.precision == BridgeCorrespondencePrecision::Exact {
                return Err(super::admission::denied(
                    BridgeCorrespondenceDenialKind::SharedSlotRequiresDeclaredWidening,
                    mapped.resolved.counters,
                ));
            }
            if !same_owner && mapped_target.widening_policy == SliceWideningPolicy::Disallow {
                return Err(super::admission::denied(
                    BridgeCorrespondenceDenialKind::SlotAlreadyOwned,
                    mapped.resolved.counters,
                ));
            }
        }
        pending_owners
            .entry(key.clone())
            .or_default()
            .insert(mapped.resolved.owner.clone());
        pending_records.push(AuthoritativeAllocationRecord {
            key: key.clone(),
            owner: mapped.resolved.owner.clone(),
            target_identity,
        });
        let mut allocation_sources = registry
            .owners
            .get(&key)
            .into_iter()
            .flat_map(|owners| owners.iter().cloned())
            .collect::<BTreeSet<_>>();
        allocation_sources.extend(
            pending_owners
                .get(&key)
                .into_iter()
                .flat_map(|owners| owners.iter().cloned()),
        );
        targets.push(InstalledCorrespondenceTarget {
            mapping_identity: mapped_target.mapping_identity,
            signal_graph_instance_id: mapped.resolved.signal_graph.graph_instance_id(),
            partition: declaration.partition,
            node: declaration.node,
            aspect,
            precision: mapped_target.precision,
            admitted_source_widening: mapped_target.admitted_source_widening,
            allocation_sources: allocation_sources.into_iter().collect(),
        });
    }
    let targets = ProvenCorrespondenceTargets::admit(targets)
        .map_err(|kind| super::admission::denied(kind, mapped.resolved.counters))?;
    Ok(PlannedCorrespondence {
        resolved: mapped.resolved,
        targets,
        pending_records,
    })
}
