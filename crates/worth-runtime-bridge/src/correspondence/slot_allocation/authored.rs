//! Read-only authored allocation meaning, without retaining installed authority.
use super::{AllocationSlotIdentity, CorrespondenceAllocationRegistry};
use crate::correspondence::BridgeSemanticClaimOrigin;
use crate::mapping::BridgeAspectRegistrationId;
use worth_signal::facade::{Aspect, NodeId, PartitionToken};

pub(crate) struct BridgeAuthoredAllocationView<'a> {
    pub(crate) graph_instance: u64,
    pub(crate) partition: &'a PartitionToken,
    pub(crate) node: NodeId,
    pub(crate) aspect: Aspect,
    pub(crate) owner: &'a BridgeSemanticClaimOrigin,
    pub(crate) mapping: &'a BridgeAspectRegistrationId,
    pub(crate) slot: AllocationSlotIdentity,
}
impl CorrespondenceAllocationRegistry {
    pub(crate) fn authored_readmission_storage_bytes(
        records: usize,
        text_bytes: usize,
    ) -> Option<u64> {
        use std::mem::size_of;
        let tree = super::super::authored_storage::tree;
        tree(size_of::<super::AuthoritativeAllocationRecord>(), records)?
            .checked_add(tree(
                size_of::<super::AllocationKey>().checked_add(size_of::<
                    std::collections::BTreeSet<BridgeSemanticClaimOrigin>,
                >())?,
                records,
            )?)?
            .checked_add(tree(size_of::<BridgeSemanticClaimOrigin>(), records)?)?
            .checked_add(tree(
                size_of::<super::AllocationTargetKey>().checked_add(size_of::<Aspect>())?,
                records,
            )?)?
            .checked_add(text_bytes.checked_mul(4)?)?
            .try_into()
            .ok()
    }
    pub(crate) fn readmit_authored_allocation(
        &mut self,
        graph_instance: u64,
        partition: PartitionToken,
        node: NodeId,
        aspect: Aspect,
        owner: worth_foundational::facade::BridgeSemanticClaimReference,
        mapping: BridgeAspectRegistrationId,
        slot: AllocationSlotIdentity,
    ) -> Result<(), super::super::BridgeCorrespondenceDenialKind> {
        let owner = BridgeSemanticClaimOrigin::Recorded(owner);
        let target_identity = super::AllocationTargetIdentity {
            aspect_registration_id: mapping,
            slot,
        };
        let target_key = super::AllocationTargetKey {
            signal_graph_instance_id: graph_instance,
            partition: partition.clone(),
            node,
            owner: owner.clone(),
            target_identity: target_identity.clone(),
        };
        if self
            .retained_aspect_by_target
            .get(&target_key)
            .is_some_and(|selected| *selected != aspect)
            || matches!(slot,super::AllocationSlotIdentity::Exact(exact) if exact!=aspect)
        {
            return Err(super::super::BridgeCorrespondenceDenialKind::InvalidPortableDependency);
        }
        self.commit(super::AuthoritativeAllocationRecord {
            key: super::AllocationKey {
                signal_graph_instance_id: graph_instance,
                partition,
                node,
                aspect,
            },
            owner,
            target_identity,
        });
        Ok(())
    }
    pub(crate) fn authored_allocations(
        &self,
    ) -> impl Iterator<Item = BridgeAuthoredAllocationView<'_>> {
        self.authoritative_records
            .iter()
            .map(|record| BridgeAuthoredAllocationView {
                graph_instance: record.key.signal_graph_instance_id,
                partition: &record.key.partition,
                node: record.key.node,
                aspect: record.key.aspect,
                owner: &record.owner,
                mapping: &record.target_identity.aspect_registration_id,
                slot: record.target_identity.slot,
            })
    }
}
