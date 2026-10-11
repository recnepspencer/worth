//! Installed branch lookup with declared branch order for visits and drops.

use std::collections::{BTreeSet, HashMap, TryReserveError};

use super::{ProductBranchIdentity, ProductBranchRegistryEntry};

#[derive(Debug, Default)]
pub(super) struct InstalledProductBranches {
    exact: HashMap<ProductBranchIdentity, ProductBranchRegistryEntry>,
    order: BTreeSet<ProductBranchIdentity>,
}

impl InstalledProductBranches {
    pub(super) fn get(&self, key: &ProductBranchIdentity) -> Option<&ProductBranchRegistryEntry> {
        self.exact.get(key)
    }

    pub(super) fn contains_key(&self, key: &ProductBranchIdentity) -> bool {
        self.exact.contains_key(key)
    }

    pub(super) fn len(&self) -> usize {
        self.exact.len()
    }

    pub(super) fn capacity(&self) -> usize {
        self.exact.capacity()
    }

    pub(super) fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.exact.try_reserve(additional)
    }

    pub(super) fn insert(&mut self, key: ProductBranchIdentity, entry: ProductBranchRegistryEntry) {
        self.order.insert(key.clone());
        self.exact.insert(key, entry);
    }

    pub(super) fn remove(
        &mut self,
        key: &ProductBranchIdentity,
    ) -> Option<ProductBranchRegistryEntry> {
        self.order.remove(key);
        self.exact.remove(key)
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &ProductBranchIdentity> {
        self.order.iter()
    }
}

impl Drop for InstalledProductBranches {
    fn drop(&mut self) {
        while let Some(key) = self.order.pop_first() {
            self.exact.remove(&key);
        }
    }
}
