//! Concrete rule/member and three touched-slice admission slots.
use crate::validation::data::CustomInvariantTouchedSummary;
use worth_execution::ExecutionMemoryReservation;
pub(crate) struct ValidationLeafMemory {
    pub(crate) length: usize,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
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
pub(crate) struct ValidationTouchedMemory {
    pub(crate) entities: ValidationLeafMemory,
    pub(crate) relations: ValidationLeafMemory,
    pub(crate) partitions: ValidationLeafMemory,
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
