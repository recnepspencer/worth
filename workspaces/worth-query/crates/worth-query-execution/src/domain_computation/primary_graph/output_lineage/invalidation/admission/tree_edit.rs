//! A sequence prepares every path but retains only its resulting ordered tree.
use std::marker::PhantomData;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::fact_key::FactPostingKey;
use super::{index_capacity, IndexAdmission, RetainedIndexAdmission};

/// One tree value's edits, which its caller never clones between them. The
/// predecessor's reservation continues to own all shared old nodes.
///
/// No counter refunds: each edit admits its preparation before it runs, and
/// an admitted byte stays charged. Until a removal, the sequence admits no
/// more than everything its insertions can allocate together, a bound fixed
/// by the tree's final size rather than its number of edits. A removal can
/// merge into a fresh node, so from then on every edit admits its own path.
pub(in super::super) struct PreparedTreeEdits<K, V> {
    /// The sum of every edit's own copied-path forecast.
    paths: u64,
    /// Preparation this sequence has admitted on the meter.
    admitted: u64,
    /// The largest length any insertion so far can leave.
    reach: usize,
    removed: bool,
    shape: PhantomData<(K, V)>,
}

impl<K, V> PreparedTreeEdits<K, V> {
    pub(in super::super) fn new() -> Self {
        Self {
            paths: 0,
            admitted: 0,
            reach: 0,
            removed: false,
            shape: PhantomData,
        }
    }

    pub(in super::super) fn insert(
        &mut self,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.ordered_read(entries)?;
        self.admit_insertion(entries, admission)
    }

    pub(in super::super) fn remove(
        &mut self,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.ordered_remove::<K, V>(entries)?;
        self.record_removal(entries)
    }

    pub(in super::super) fn key_insert(
        &mut self,
        key: &FactPostingKey,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.key_read(key, entries)?;
        self.admit_insertion(entries, admission)
    }

    pub(in super::super) fn key_remove(
        &mut self,
        key: &FactPostingKey,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.key_remove::<K, V>(key, entries)?;
        self.record_removal(entries)
    }

    /// New surviving nodes fit both the sum of prepared paths and the whole
    /// final stable tree. Intermediate roots are preparation, not history.
    /// An unchanged tree contributes nothing; a changed empty tree owns a root.
    pub(in super::super) fn retain(
        self,
        entries: usize,
        admission: &mut impl RetainedIndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        if self.paths == 0 {
            return Ok(());
        }
        let whole = index_capacity::retained_map_bytes::<K, V>(entries)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.record_index_bytes(self.paths.min(whole))
    }

    /// Admits what this insertion raises the sequence's allocation bound to,
    /// before the edit runs. O(1); navigation was admitted by the caller.
    fn admit_insertion(
        &mut self,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        // A length whose path has a forecast also has a successor length.
        let path = self.add_path(entries)?;
        let bound = if self.removed {
            self.admitted.checked_add(path)
        } else {
            self.reach = self.reach.max(entries.saturating_add(1));
            index_capacity::insertion_sequence_bytes::<K, V>(self.reach)
                .map(|whole| whole.min(self.paths))
        };
        // Both bounds only grow, so the increment is never negative.
        let increment = bound
            .and_then(|bound| bound.checked_sub(self.admitted))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.bytes(increment)?;
        self.admitted += increment;
        Ok(())
    }

    /// The caller's removal admission already charged this edit's own path.
    fn record_removal(&mut self, entries: usize) -> Result<(), CompanionPreflightStop> {
        let path = self.add_path(entries)?;
        self.admitted = self
            .admitted
            .checked_add(path)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        self.removed = true;
        Ok(())
    }

    fn add_path(&mut self, entries: usize) -> Result<u64, CompanionPreflightStop> {
        let path = index_capacity::ordered_edit_bytes::<K, V>(entries)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        self.paths = self
            .paths
            .checked_add(path)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        Ok(path)
    }
}

#[cfg(test)]
mod allocation_tests;
#[cfg(test)]
mod tests;
