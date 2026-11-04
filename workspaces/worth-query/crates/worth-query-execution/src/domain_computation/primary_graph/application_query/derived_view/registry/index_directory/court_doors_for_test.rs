//! Sealed test doors over this module's private members.
use super::*;

impl ManagedIndexDirectory {
    #[doc(hidden)]
    pub fn dispose_for_test(self) {
        Self::dispose(self)
    }
}

impl ManagedIndexDirectory {
    #[doc(hidden)]
    pub fn get_for_test(&self, key: &Key) -> Option<(u64, &RegisteredIndex)> {
        Self::get(self, key)
    }
}

impl ManagedIndexDirectory {
    #[doc(hidden)]
    pub fn insert_for_test(&mut self, id: u64, entry: RegisteredIndex) {
        Self::insert(self, id, entry)
    }
}

impl ManagedIndexDirectory {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}

impl RegisteredIndex {
    #[doc(hidden)]
    pub fn prepare_for_test(
        execution: &ExecutionRequest<'_, '_>,
        key: Key,
        state: Arc<dyn Any + Send + Sync>,
        cleanup: Arc<dyn RegisteredManagedIndex>,
    ) -> Result<Self, Cause> {
        Self::prepare(execution, key, state, cleanup)
    }
}

impl ManagedIndexDirectory {
    #[doc(hidden)]
    pub fn remove_for_test(&mut self, id: u64) -> Option<RetiredIndex> {
        Self::remove(self, id)
    }
}
