//! Copy work for the two owned partition override trees.
use super::PartitionVersionOverrides;
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;
impl PartitionVersionOverrides {
    pub(crate) fn admit_clone_work(
        &self,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(
            self.partitions
                .len()
                .checked_add(self.details.len())
                .and_then(|n| n.checked_mul(128 + std::mem::size_of::<super::AspectVersion>()))
                .and_then(|n| n.checked_add(64)),
        )?;
        for partition in self.partitions.keys() {
            work.reserve(Some(partition.0.len()))?;
        }
        for scope in self.details.keys() {
            work.reserve(
                scope
                    .partition
                    .0
                    .len()
                    .checked_add(scope.detail.as_ref().map_or(0, String::len)),
            )?;
        }
        Ok(())
    }
}
