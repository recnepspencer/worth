//! Copy work for the owned scope-path version tree.
use super::PartitionVersionOverrides;
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;
impl PartitionVersionOverrides {
    pub(crate) fn admit_clone_work(
        &self,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<(), SignalError> {
        work.reserve(
            self.paths
                .len()
                .checked_mul(128 + std::mem::size_of::<super::PathVersions>())
                .and_then(|n| n.checked_add(64)),
        )?;
        for path in self.paths.keys() {
            work.reserve(
                path.depth()
                    .checked_mul(std::mem::size_of::<String>())
                    .and_then(|bytes| bytes.checked_add(path.total_segment_bytes())),
            )?;
        }
        Ok(())
    }
}
