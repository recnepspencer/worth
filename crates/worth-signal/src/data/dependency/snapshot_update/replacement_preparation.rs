//! Replacement snapshot normalization before commit preparation.
use super::{DependencySnapshot, ReplacementSnapshotUpdate, SharedDependencySnapshot};
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;

impl ReplacementSnapshotUpdate {
    pub(crate) fn from_snapshot_with_work(
        snapshot: DependencySnapshot,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Self, SignalError> {
        let snapshot = snapshot.canonicalize_with_work(work)?;
        Ok(Self {
            snapshot: SharedDependencySnapshot::new(snapshot),
        })
    }
}

#[cfg(test)]
mod tests;
