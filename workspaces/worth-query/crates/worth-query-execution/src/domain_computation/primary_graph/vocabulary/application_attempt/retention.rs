//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow::instance::progression::retention`: the items below and every impl they own.
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph) const SHARD_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorkflowInstanceProgressKey {
    pub(in crate::domain_computation::primary_graph) branch_occurrence: u64,
    instance: EntityId,
    definition: EntityId,
}

impl WorkflowInstanceProgressKey {
    pub(in crate::domain_computation::primary_graph) const fn new(
        branch_occurrence: u64,
        instance: EntityId,
        definition: EntityId,
    ) -> Self {
        Self {
            branch_occurrence,
            instance,
            definition,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn shard_index(self) -> usize {
        let mixed = self.branch_occurrence.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ self.instance.local_slot_value()
            ^ self.definition.local_slot_value().rotate_left(23);
        mixed as usize % SHARD_COUNT
    }
}
