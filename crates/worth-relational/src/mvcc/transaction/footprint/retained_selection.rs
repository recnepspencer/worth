//! A retained selection writes one full-GEN record under the current basis.
use super::{
    Denial, RelationalTransactionFootprint, RelationalTransactionReadLocus as Read,
    RelationalTransactionWriteLocus as Write,
};
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::RecordRef;

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
