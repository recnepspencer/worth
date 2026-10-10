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
        // The opaque lease identity is issued from the owner's checked, monotonic
        // ordinal under its registry lock; replacement removed its prior position.
        self.order.insert(position, key.clone());
        self.exact.insert(key, entry);
        drop(prior);
    }

    pub(super) fn remove(&mut self, key: &ExactComponentBasisKey) -> Option<PinEntry> {
        let entry = self.exact.remove(key)?;
        self.order
            .remove(&(entry.basis_order.clone(), entry.lease_identity));
        Some(entry)
    }

    #[cfg(test)]
    pub(super) fn copied_encoding_bytes(&self, key: &ExactComponentBasisKey) -> usize {
        let entry = self.exact.get(key).unwrap();
        let indexed = self
            .order
            .iter()
            .find(|(_, exact)| *exact == key)
            .unwrap()
            .0;
        let encoding = |order: &ComponentBasisOrder| match order {
            ComponentBasisOrder::Relational { reference, .. }
            | ComponentBasisOrder::Signal { reference } => (reference.as_ptr(), reference.len()),
        };
        let (pointer, bytes) = encoding(&entry.basis_order);
        if pointer == encoding(&indexed.0).0 {
            0
        } else {
            bytes
        }
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
