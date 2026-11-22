mod entry_tree;
mod storage_geometry;

use super::AuthoritativeStateAdmissionDenial;
use crate::aspects::allocation_visitor::FoundationalAllocationVisitor;
use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;
use entry_tree::AspectEntryTree;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct CanonicalAspectStateMap {
    pub(super) entries: AspectEntryTree,
    #[serde(skip)]
    allocation_bytes: u128,
}

impl CanonicalAspectStateMap {
    pub(in crate::aspects) fn payload_storage_bytes(&self) -> u128 {
        self.allocation_bytes
    }

    pub(crate) fn from_validated_entries(
        entries: impl IntoIterator<Item = ContractValidatedAspectValue>,
    ) -> Result<Self, AuthoritativeStateAdmissionDenial> {
        let mut map = Self::empty();
        for entry in entries {
            if map.entries.contains_key(entry.key()) {
                return Err(AuthoritativeStateAdmissionDenial::DuplicateAspectKey(
                    entry.key().clone(),
                ));
            }
            map.insert_value(entry);
        }
        Ok(map)
    }

    pub(crate) fn from_sorted_values(
        values: impl ExactSizeIterator<Item = ContractValidatedAspectValue>,
    ) -> Self {
        let mut allocation_bytes = 0_u128;
        let entries = AspectEntryTree::from_sorted_values(values.map(|value| {
            allocation_bytes += entry_bytes(&value);
            value
        }));
        Self {
            entries,
            allocation_bytes,
        }
    }

    fn empty() -> Self {
        Self {
            entries: AspectEntryTree::new(),
            allocation_bytes: 0,
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&AspectKey, &ContractValidatedAspectValue)> {
        self.entries.iter()
    }

    pub fn get(&self, key: &AspectKey) -> Option<&ContractValidatedAspectValue> {
        self.entries.get(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Cached reachable payload, including shared keys, contracts and values,
    /// plus exact persistent AVL node backing. Not a physical clone quote.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        let nodes = (self.len() as u128) * (AspectEntryTree::node_allocation_bytes() as u128);
        self.allocation_bytes
            .saturating_add(nodes)
            .min(usize::MAX as u128) as usize
    }

    /// Visits every distinct physical heap allocation reachable from this map.
    pub fn visit_allocations(
        &self,
        ancestor_unique: bool,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        self.entries.visit_allocations(ancestor_unique, visitor);
    }

    /// Visits physical heap allocations reachable from `self` that were not
    /// already present in `previous`.
    pub fn visit_new_allocations(
        &self,
        previous: Option<&Self>,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        match previous {
            Some(previous) => self
                .entries
                .visit_new_allocations(&previous.entries, visitor),
            None => self.entries.visit_allocations(false, visitor),
        }
    }
}

impl CanonicalAspectStateMap {
    pub(crate) fn remove(&mut self, key: &AspectKey) {
        if let Some(removed) = self.entries.remove(key) {
            self.allocation_bytes -= entry_bytes(&removed);
        }
    }

    pub(crate) fn insert_value(&mut self, value: ContractValidatedAspectValue) {
        let added = entry_bytes(&value);
        if let Some(previous) = self.entries.insert(value) {
            self.allocation_bytes -= entry_bytes(&previous);
        }
        self.allocation_bytes += added;
    }
}

fn entry_bytes(value: &ContractValidatedAspectValue) -> u128 {
    value.owned_allocation_capacity_bytes() as u128
}

impl PartialEq for CanonicalAspectStateMap {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl Eq for CanonicalAspectStateMap {}

impl std::fmt::Debug for CanonicalAspectStateMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalAspectStateMap")
            .field("entries", &self.entries)
            .finish()
    }
}

impl CanonicalAspectStateMap {
    /// Checked declared upper bound of the backing reachable from one holder:
    /// persistent AVL nodes, keys, values and contracts. Backing shared with
    /// another holder is included in full for each holder. Every step is
    /// checked: `None` means the bound is unrepresentable.
    pub fn retained_storage_bytes(&self) -> Option<usize> {
        usize::try_from(self.allocation_bytes)
            .ok()?
            .checked_add(storage_geometry::retained_nodes(self.len())?)
    }

    /// Exact persistent-tree node backing for `entries` aspects, excluding
    /// nested keys/values. An empty map allocates zero nodes.
    pub fn construction_storage_bytes(entries: usize) -> Option<usize> {
        storage_geometry::retained_nodes(entries)
    }

    /// Peak incremental construction scratch above independently retained input
    /// and the final persistent nodes when inserting an arbitrary unsorted
    /// stream one entry at a time.
    pub fn construction_scratch_bytes(entries: usize, _maximum_key_width: usize) -> Option<usize> {
        if entries == 0 {
            return Some(0);
        }
        storage_geometry::edit_paths(entries, 1)
    }

    /// Conservative copy-path scratch for this ordered edit batch.
    pub(crate) fn edit_storage_bytes(
        &self,
        operations: usize,
        additional_entries: usize,
        _new_key_width: usize,
    ) -> Option<usize> {
        let entries = self.len().checked_add(additional_entries)?;
        storage_geometry::edit_paths(entries, operations)
    }
}

#[cfg(test)]
mod shared_holder_bound_tests;
#[cfg(test)]
mod tests;
