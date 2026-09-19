//! Work admission for owned shape keys derived from an immutable snapshot.
use super::DependencySnapshot;
use crate::data::dependency::DependencySnapshotShape;
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;
impl DependencySnapshot {
    pub(crate) fn shape_with_work(
        &self,
        work: &mut EvaluationWork<'_>,
    ) -> Result<DependencySnapshotShape, SignalError> {
        work.reserve(
            self.entries()
                .len()
                .checked_mul(std::mem::size_of::<crate::data::dependency::DependencySortKey>() + 1)
                .and_then(|n| n.checked_add(32))
                .filter(|n| *n <= isize::MAX as usize),
        )?;
        for entry in self.entries() {
            // Copy into shape keys and compare adjacent keys in its constructor.
            let bytes = entry.scope.as_ref().map_or(Some(0), |s| {
                s.partition
                    .0
                    .len()
                    .checked_add(s.detail.as_ref().map_or(0, String::len))
            });
            work.reserve(
                bytes
                    .and_then(|n| n.checked_mul(3))
                    .and_then(|n| n.checked_add(32)),
            )?;
        }
        Ok(self.shape())
    }
}
