//! Copy admission for an immutable snapshot ready for storage publication.
use super::{CommittedSnapshotUpdate, DependencySnapshot, SharedDependencySnapshot};
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;

impl CommittedSnapshotUpdate {
    pub(crate) fn materialize_with_work(
        &self,
        previous: &DependencySnapshot,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<SharedDependencySnapshot, SignalError> {
        match self {
            Self::Replace(replacement) => {
                work.reserve(Some(1))?;
                Ok(replacement.snapshot().clone())
            }
            Self::VersionOnly(update) => {
                let entries = previous.entries();
                if entries.len() != update.versions().len() {
                    return Err(SignalError::invalid_input(
                        "snapshot version count does not match its shape",
                    ));
                }
                work.reserve(
                    entries
                        .len()
                        .checked_mul(
                            std::mem::size_of::<crate::data::dependency::DependencySnapshotEntry>()
                                + 2,
                        )
                        .and_then(|n| n.checked_add(32))
                        .filter(|n| *n <= isize::MAX as usize),
                )?;
                for entry in entries {
                    let bytes = entry
                        .scope
                        .as_ref()
                        .map_or(Some(0), |s| s.path().checked_segment_bytes());
                    work.reserve(bytes)?;
                }
                Ok(SharedDependencySnapshot::new(
                    previous.with_updated_versions(update.versions().as_slice()),
                ))
            }
        }
    }
}
