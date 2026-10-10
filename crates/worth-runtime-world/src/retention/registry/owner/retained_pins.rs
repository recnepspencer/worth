//! Exact pin lookup and visits by admitted basis and owner-issued lease position.

use std::collections::{BTreeMap, HashMap};

use super::{ComponentBasisLeaseIdentity, ComponentBasisOrder, ExactComponentBasisKey, PinEntry};

#[derive(Default)]
pub(super) struct RetainedComponentPins {
    exact: HashMap<ExactComponentBasisKey, PinEntry>,
    order: BTreeMap<(ComponentBasisOrder, ComponentBasisLeaseIdentity), ExactComponentBasisKey>,
}

impl RetainedComponentPins {
    pub(super) fn get(&self, key: &ExactComponentBasisKey) -> Option<&PinEntry> {
        self.exact.get(key)
    }

    pub(super) fn get_mut(&mut self, key: &ExactComponentBasisKey) -> Option<&mut PinEntry> {
        self.exact.get_mut(key)
    }

    pub(super) fn contains_key(&self, key: &ExactComponentBasisKey) -> bool {
        self.exact.contains_key(key)
    }

    pub(super) fn insert(&mut self, key: ExactComponentBasisKey, entry: PinEntry) {
        let prior = self.remove(&key);
        let position = (entry.basis_order.clone(), entry.lease_identity);
        assert!(
            self.order.insert(position, key.clone()).is_none(),
            "lease position is unique"
        );
        self.exact.insert(key, entry);
        drop(prior);
    }

    pub(super) fn remove(&mut self, key: &ExactComponentBasisKey) -> Option<PinEntry> {
        let entry = self.exact.remove(key)?;
        self.order
            .remove(&(entry.basis_order.clone(), entry.lease_identity));
        Some(entry)
    }

    pub(super) fn by_declared_basis(
        &self,
    ) -> impl Iterator<Item = (&ExactComponentBasisKey, &PinEntry)> {
        self.order.values().map(|key| {
            (
                key,
                self.exact
                    .get(key)
                    .expect("ordered position references its retained pin"),
            )
        })
    }
}

impl Drop for RetainedComponentPins {
    fn drop(&mut self) {
        // Owner leases also leave in declared order when the registry drops.
        while let Some((_, key)) = self.order.pop_first() {
            self.exact.remove(&key);
        }
    }
}
