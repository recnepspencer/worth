use super::{
    index_row::IndexRow, staging_storage::Author, RelationalTransactionFootprint,
    RelationalTransactionReadLocus, RelationalTransactionStagingDenial as Denial,
};
use crate::transactions::data::WorkerIntentBatch;
use worth_execution::ExecutionAllocationPolicy;

impl RelationalTransactionFootprint {
    pub(crate) fn admit_read(
        &mut self,
        locus: RelationalTransactionReadLocus,
    ) -> Result<(), Denial> {
        self.record_read(locus, ExecutionAllocationPolicy::SystemAllocation)
    }
    pub(super) fn for_staged_batch(
        &self,
        batch: &WorkerIntentBatch,
        batch_index: usize,
        policy: ExecutionAllocationPolicy<'_, '_>,
        mut emit: impl FnMut(IndexRow) -> Result<(), Denial>,
    ) -> Result<Self, Denial> {
        let mut reads = Author::new(&self.reads, policy)?;
        let mut writes = Author::new(&self.writes, policy)?;
        let mut partitions = Author::new(&self.write_partitions, policy)?;
        super::overlay_indexing::index_batch(batch, batch_index, |row| {
            if let Some(read) = row.read() {
                reads.insert(read)?;
            }
            if let Some(write) = row.write() {
                writes.insert(write)?;
            }
            if let Some(partition) = row.partition() {
                partitions.insert(partition)?;
            }
            emit(row)
        })?;
        let reads = reads.finish()?;
        let writes = writes.finish()?;
        let write_partitions = partitions.finish()?;
        Ok(Self {
            basis: self.basis.clone(),
            reads,
            writes,
            write_partitions,
        })
    }
}
