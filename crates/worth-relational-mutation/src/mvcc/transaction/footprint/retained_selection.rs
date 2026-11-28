//! A retained selection writes one full-GEN record under the current basis.
use super::{
    Denial, RelationalTransactionFootprint, RelationalTransactionReadLocus as Read,
    RelationalTransactionWriteLocus as Write,
};
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::transactions::data::primitives::RecordRef;

impl RelationalTransactionFootprint {
    pub(crate) fn record_retained_entity_selection(
        &mut self,
        entity: EntityId,
        kind: KindId,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        let record = RecordRef::Entity(entity);
        let reads = self.reads.with_inserted(
            [
                Ok(Read::Existing(record.clone())),
                Ok(Read::EntitySchema(kind)),
            ],
            policy,
        )?;
        let writes = self
            .writes
            .with_inserted(std::iter::once(Ok(Write::Existing(record))), policy)?;
        let partitions = self
            .write_partitions
            .with_inserted(std::iter::once(Ok(entity.partition_id)), policy)?;
        self.reads = reads;
        self.writes = writes;
        self.write_partitions = partitions;
        Ok(())
    }
}

#[doc(hidden)]
pub trait RelationalTransactionFootprintOwnerPort: Sized {
    fn record_retained_entity_selection(
        &mut self,
        entity: EntityId,
        kind: KindId,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial>;
}

impl RelationalTransactionFootprintOwnerPort for RelationalTransactionFootprint {
    fn record_retained_entity_selection(
        &mut self,
        entity: EntityId,
        kind: KindId,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        self.record_retained_entity_selection(entity, kind, policy)
    }
}
