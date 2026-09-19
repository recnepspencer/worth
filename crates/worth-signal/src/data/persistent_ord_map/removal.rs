//! Removal without materializing a value that the caller will discard.
use super::{
    base_value_if_live, entry_handle, record_base_retirement, PersistentOrdMap,
    PersistentOrdMapStorage,
};

impl<K: Clone + Ord, V: Clone> PersistentOrdMap<K, V> {
    /// Retires the key without cloning its payload from retained storage.
    /// Like other raw mutations, this invalidates the carried retained charge.
    pub(crate) fn remove_discard(&mut self, key: &K) -> bool {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.remove(key).is_some(),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                len,
            } => {
                let present = entry_handle::get(changes, key).is_some()
                    || base_value_if_live(base, retired_base_intervals, key).is_some();
                if present {
                    *len -= 1;
                    entry_handle::remove(changes, key);
                    if base.contains_key(key) {
                        record_base_retirement(base, retired_base_intervals, key);
                    }
                }
                present
            }
        }
    }
}

#[cfg(test)]
mod tests;
