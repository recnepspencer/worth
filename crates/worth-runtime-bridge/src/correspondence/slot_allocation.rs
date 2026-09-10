use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use worth_signal::facade::{Aspect, PartitionToken, MAX_ASPECTS};

use crate::mapping::BridgeAspectRegistrationId;

use super::{
    BridgeSignalAspectTargetDeclaration, BridgeSignalSlotRequest, InstalledCorrespondenceTarget,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct AllocationKey {
    pub(super) signal_graph_instance_id: u64,
    pub(super) partition: PartitionToken,
    pub(super) node: worth_signal::facade::NodeId,
    pub(super) aspect: Aspect,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct AuthoritativeAllocationRecord {
    pub(super) key: AllocationKey,
    pub(super) owner: String,
    pub(super) target_identity: AllocationTargetIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct AllocationTargetIdentity {
    aspect_registration_id: BridgeAspectRegistrationId,
    slot: AllocationSlotIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AllocationSlotIdentity {
    Allocate,
    Exact(Aspect),
}

pub(super) fn allocation_target_identity(
    declaration: &BridgeSignalAspectTargetDeclaration,
) -> AllocationTargetIdentity {
    let slot = match &declaration.slot {
        BridgeSignalSlotRequest::Allocate => AllocationSlotIdentity::Allocate,
        BridgeSignalSlotRequest::Exact(aspect) => AllocationSlotIdentity::Exact(aspect.aspect()),
    };
    AllocationTargetIdentity {
        aspect_registration_id: declaration.aspect_registration_id.clone(),
        slot,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AllocationTargetKey {
    signal_graph_instance_id: u64,
    partition: PartitionToken,
    node: worth_signal::facade::NodeId,
    owner: String,
    target_identity: AllocationTargetIdentity,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CorrespondenceAllocationRegistry {
    pub(super) authoritative_records: BTreeSet<AuthoritativeAllocationRecord>,
    pub(super) owners: BTreeMap<AllocationKey, BTreeSet<String>>,
    retained_aspect_by_target: BTreeMap<AllocationTargetKey, Aspect>,
}

impl CorrespondenceAllocationRegistry {
    pub(crate) fn admits_source_set(&self, target: &InstalledCorrespondenceTarget) -> bool {
        let key = AllocationKey {
            signal_graph_instance_id: target.signal_graph_instance_id,
            partition: target.partition.clone(),
            node: target.node,
            aspect: target.aspect,
        };
        self.owners
            .get(&key)
            .is_some_and(|owners| owners.iter().eq(target.allocation_sources.iter()))
    }

    pub(super) fn commit(&mut self, record: AuthoritativeAllocationRecord) {
        self.owners
            .entry(record.key.clone())
            .or_default()
            .insert(record.owner.clone());
        self.retained_aspect_by_target.insert(
            AllocationTargetKey {
                signal_graph_instance_id: record.key.signal_graph_instance_id,
                partition: record.key.partition.clone(),
                node: record.key.node,
                owner: record.owner.clone(),
                target_identity: record.target_identity.clone(),
            },
            record.key.aspect,
        );
        self.authoritative_records.insert(record);
    }

    fn from_authoritative_records(
        authoritative_records: BTreeSet<AuthoritativeAllocationRecord>,
    ) -> Self {
        let mut rebuilt = Self::default();
        for record in authoritative_records {
            rebuilt.commit(record);
        }
        rebuilt
    }

    pub(crate) fn reconstruct_derived_indexes(&self) -> Self {
        Self::from_authoritative_records(self.authoritative_records.clone())
    }

    #[cfg(test)]
    pub(crate) fn destroy_derived_indexes(&mut self) {
        self.owners.clear();
        self.retained_aspect_by_target.clear();
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.authoritative_records.is_empty()
            && self.owners.is_empty()
            && self.retained_aspect_by_target.is_empty()
    }
}

pub(crate) type SharedCorrespondenceAllocationRegistry =
    Arc<RwLock<CorrespondenceAllocationRegistry>>;

pub(super) enum SlotAllocation {
    Allocated(Aspect),
    CapacityExhausted,
    NodeContractMismatch,
}

pub(super) fn allocate_slot(
    registry: &CorrespondenceAllocationRegistry,
    pending: &BTreeMap<AllocationKey, BTreeSet<String>>,
    signal_graph_instance_id: u64,
    declaration: &BridgeSignalAspectTargetDeclaration,
    owner: &str,
    target_identity: &AllocationTargetIdentity,
) -> (SlotAllocation, usize) {
    match &declaration.slot {
        BridgeSignalSlotRequest::Exact(aspect) => (
            if signal_node_admits(declaration, aspect.aspect()) {
                SlotAllocation::Allocated(aspect.aspect())
            } else {
                SlotAllocation::NodeContractMismatch
            },
            1,
        ),
        BridgeSignalSlotRequest::Allocate => {
            let retained_key = AllocationTargetKey {
                signal_graph_instance_id,
                partition: declaration.partition.clone(),
                node: declaration.node,
                owner: owner.to_string(),
                target_identity: target_identity.clone(),
            };
            if let Some(aspect) = registry.retained_aspect_by_target.get(&retained_key) {
                let key = AllocationKey {
                    signal_graph_instance_id,
                    partition: declaration.partition.clone(),
                    node: declaration.node,
                    aspect: *aspect,
                };
                if signal_node_admits(declaration, *aspect) && !pending.contains_key(&key) {
                    return (SlotAllocation::Allocated(*aspect), 1);
                }
            }
            let mut examined = 0;
            let mut admitted_slot_exists = false;
            let aspect = (0..MAX_ASPECTS as u8).map(Aspect::new).find(|aspect| {
                examined += 1;
                let admitted = signal_node_admits(declaration, *aspect);
                admitted_slot_exists |= admitted;
                let key = AllocationKey {
                    signal_graph_instance_id,
                    partition: declaration.partition.clone(),
                    node: declaration.node,
                    aspect: *aspect,
                };
                admitted && !registry.owners.contains_key(&key) && !pending.contains_key(&key)
            });
            (
                aspect.map_or_else(
                    || {
                        if admitted_slot_exists {
                            SlotAllocation::CapacityExhausted
                        } else {
                            SlotAllocation::NodeContractMismatch
                        }
                    },
                    SlotAllocation::Allocated,
                ),
                examined,
            )
        }
    }
}

pub(super) fn signal_node_admits(
    declaration: &BridgeSignalAspectTargetDeclaration,
    aspect: Aspect,
) -> bool {
    let contract = declaration.node_capability.contract();
    let slot = worth_signal::facade::AspectMask::from_aspect(aspect);
    let partition_admitted = |scopes: &Option<Vec<worth_signal::facade::PartitionSubscription>>| {
        scopes.as_ref().is_none_or(|scopes| {
            scopes
                .iter()
                .any(|scope| scope.partition == declaration.partition)
        })
    };
    contract.semantics.reads.contains(slot)
        && contract.projection.consumes.contains(slot)
        && partition_admitted(&contract.semantics.partition_scope)
        && partition_admitted(&contract.projection.consumes_partitions)
}
