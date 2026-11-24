//! Concrete rule/member and three touched-slice admission slots.
use crate::internal_port::validation::data::custom_rule::touched_scope::touched_ids::entity_ids::InvariantEntityIdsOwnerPort as _;
use crate::internal_port::validation::data::custom_rule::touched_scope::touched_ids::partition_ids::InvariantPartitionIdsOwnerPort as _;
use crate::internal_port::validation::data::custom_rule::touched_scope::touched_ids::relation_ids::InvariantRelationIdsOwnerPort as _;
use crate::validation::data::custom_rule::touched_scope::planned_records::CustomInvariantTouchedSummary;
use worth_execution::ExecutionMemoryReservation;
pub struct ValidationLeafMemory {
    pub length: usize,
    pub memory: Option<ExecutionMemoryReservation>,
}
impl ValidationLeafMemory {
    pub(super) fn install(
        &mut self,
        length: usize,
        install: impl FnOnce(
            Option<ExecutionMemoryReservation>,
        ) -> Result<(), Option<ExecutionMemoryReservation>>,
    ) -> Result<(), &'static str> {
        if length != self.length {
            return Err("validation leaf admission length mismatch");
        }
        if let Err(unused) = install(self.memory.take()) {
            drop(unused);
        }
        Ok(())
    }
}
pub struct ValidationTouchedMemory {
    pub entities: ValidationLeafMemory,
    pub relations: ValidationLeafMemory,
    pub partitions: ValidationLeafMemory,
}
impl ValidationTouchedMemory {
    pub(super) fn install(
        &mut self,
        actual: &CustomInvariantTouchedSummary,
    ) -> Result<(), &'static str> {
        self.entities
            .install(actual.visible_entity_ids.len(), |grant| {
                actual.visible_entity_ids.retain_decoded_memory(grant)
            })?;
        self.relations
            .install(actual.visible_relation_ids.len(), |grant| {
                actual.visible_relation_ids.retain_decoded_memory(grant)
            })?;
        self.partitions
            .install(actual.touched_partition_ids.len(), |grant| {
                actual.touched_partition_ids.retain_decoded_memory(grant)
            })
    }
}
