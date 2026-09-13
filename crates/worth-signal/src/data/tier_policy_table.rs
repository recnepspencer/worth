use crate::data::tier::TierPolicy;

/// Deterministic compact tier policy table for small cardinalities.
#[derive(Debug, Clone)]
pub struct TierPolicyTable<T: Copy + Ord> {
    entries: crate::data::persistent_vector::PersistentVector<(T, TierPolicy<T>)>,
}

impl<T: Copy + Ord> TierPolicyTable<T> {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            entries: self.entries.fork_persistent(),
        }
    }

    /// Create an empty table.
    pub fn new() -> Self {
        Self {
            entries: crate::data::persistent_vector::PersistentVector::new(),
        }
    }

    /// Insert or replace one tier policy.
    pub fn set(&mut self, policy: TierPolicy<T>) {
        match self
            .entries
            .binary_search_by_key(&policy.tier, |(tier, _)| *tier)
        {
            Ok(index) => self.entries[index] = (policy.tier, policy),
            Err(index) => self.entries.insert(index, (policy.tier, policy)),
        }
    }

    /// Read one policy by tier key.
    pub fn get(&self, tier: T) -> Option<&TierPolicy<T>> {
        self.entries
            .binary_search_by_key(&tier, |(entry_tier, _)| *entry_tier)
            .ok()
            .map(|index| &self.entries[index].1)
    }

    /// Deterministic iteration over all policies.
    pub fn iter(&self) -> impl Iterator<Item = &TierPolicy<T>> {
        self.entries.iter().map(|(_, policy)| policy)
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.entries.shares_storage_with(&other.entries)
    }
}

impl<T: Copy + Ord> Default for TierPolicyTable<T> {
    fn default() -> Self {
        Self::new()
    }
}
