//! A sequence prepares every path but retains only its resulting ordered tree.
use std::marker::PhantomData;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::fact_key::FactPostingKey;
use super::{index_capacity, IndexAdmission, RetainedIndexAdmission};

/// No counter refunds: operation preparation remains on the original meter.
/// This local forecast becomes retained storage only after its final tree is
/// known. The predecessor's reservation continues to own all shared old nodes.
pub(in super::super) struct PreparedTreeEdits<K, V> {
    paths: u64,
    shape: PhantomData<(K, V)>,
}

impl<K, V> PreparedTreeEdits<K, V> {
    pub(in super::super) fn new() -> Self {
        Self {
            paths: 0,
            shape: PhantomData,
        }
    }

    pub(in super::super) fn insert(
        &mut self,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.ordered_edit::<K, V>(entries)?;
        self.add(index_capacity::ordered_edit_bytes::<K, V>(entries))
    }

    pub(in super::super) fn remove(
        &mut self,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.ordered_remove::<K, V>(entries)?;
        self.add(index_capacity::ordered_edit_bytes::<K, V>(entries))
    }

    pub(in super::super) fn key_insert(
        &mut self,
        key: &FactPostingKey,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.key_edit::<K, V>(key, entries)?;
        self.add(index_capacity::ordered_edit_bytes::<K, V>(entries))
    }

    pub(in super::super) fn key_remove(
        &mut self,
        key: &FactPostingKey,
        entries: usize,
        admission: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        admission.key_remove::<K, V>(key, entries)?;
        self.add(index_capacity::ordered_edit_bytes::<K, V>(entries))
    }

    fn add(&mut self, bytes: Option<u64>) -> Result<(), CompanionPreflightStop> {
        self.paths = bytes
            .and_then(|bytes| self.paths.checked_add(bytes))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        Ok(())
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
}
