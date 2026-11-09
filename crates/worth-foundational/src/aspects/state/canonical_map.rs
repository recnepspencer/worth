use im::OrdMap;
mod storage_geometry;

use super::AuthoritativeStateAdmissionDenial;
use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct CanonicalAspectStateMap {
    pub(super) entries: OrdMap<AspectKey, ContractValidatedAspectValue>,
    #[serde(skip)]
    allocation_bytes: u128,
    #[serde(skip)]
    key_width_counts: OrdMap<usize, usize>,
}

impl CanonicalAspectStateMap {
    pub(in crate::aspects) fn payload_storage_bytes(&self) -> u128 {
        self.allocation_bytes
    }

    pub(crate) fn from_validated_entries(
        entries: impl IntoIterator<Item = ContractValidatedAspectValue>,
    ) -> Result<Self, AuthoritativeStateAdmissionDenial> {
        let mut canonical_entries = OrdMap::new();
        for entry in entries {
            let key = entry.key().clone();
            if canonical_entries.insert(key.clone(), entry).is_some() {
                return Err(AuthoritativeStateAdmissionDenial::DuplicateAspectKey(key));
            }
        }

        Ok(Self::from_canonical_entries(canonical_entries))
    }

    pub(crate) fn from_canonical_entries(
        entries: OrdMap<AspectKey, ContractValidatedAspectValue>,
    ) -> Self {
        let allocation_bytes = entries
            .iter()
            .map(|(key, value)| entry_bytes(key, value))
            .sum();
        let mut key_width_counts = OrdMap::new();
        for key in entries.keys() {
            let width = key.as_str().len();
            let count = key_width_counts.get(&width).copied().unwrap_or(0usize);
            key_width_counts.insert(width, count + 1);
        }
        Self {
            entries,
            allocation_bytes,
            key_width_counts,
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&AspectKey, &ContractValidatedAspectValue)> {
        self.entries.iter()
    }

    pub fn get(&self, key: &AspectKey) -> Option<&ContractValidatedAspectValue> {
        self.entries.get(key)
    }

    #[cfg(test)]
    pub(crate) fn cloned_entries(&self) -> OrdMap<AspectKey, ContractValidatedAspectValue> {
        self.entries.clone()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Cached reachable payload, including shared keys, contracts and values.
    /// Allocator/tree bookkeeping is excluded; this is not a clone quote.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.allocation_bytes.min(usize::MAX as u128) as usize
    }
}

impl CanonicalAspectStateMap {
    pub(crate) fn remove(&mut self, key: &AspectKey) {
        if let Some((key, value)) = self.entries.get_key_value(key) {
            self.allocation_bytes -= entry_bytes(key, value);
            let width = key.as_str().len();
            let count = self
                .key_width_counts
                .get(&width)
                .copied()
                .expect("each actual entry contributes its current key width");
            if count == 1 {
                self.key_width_counts.remove(&width);
            } else {
                self.key_width_counts.insert(width, count - 1);
            }
        }
        self.entries.remove(key);
    }
    pub(crate) fn insert(&mut self, key: AspectKey, value: ContractValidatedAspectValue) {
        self.remove(&key);
        self.allocation_bytes += entry_bytes(&key, &value);
        let width = key.as_str().len();
        let count = self.key_width_counts.get(&width).copied().unwrap_or(0usize);
        self.key_width_counts.insert(width, count + 1);
        self.entries.insert(key, value);
    }
}
fn entry_bytes(key: &AspectKey, value: &ContractValidatedAspectValue) -> u128 {
    std::mem::size_of::<(AspectKey, ContractValidatedAspectValue)>() as u128
        + key.owned_allocation_capacity_bytes() as u128
        + value.owned_allocation_capacity_bytes() as u128
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
    /// Checked reachable bound for one independent retained storage holder.
    /// Shared backing is included; this is deliberately not a physical Clone bill.
    pub fn retained_storage_bytes(&self) -> Option<usize> {
        usize::try_from(self.allocation_bytes)
            .ok()?
            .checked_add(storage_geometry::retained_nodes::<
                AspectKey,
                ContractValidatedAspectValue,
            >(self.len())?)?
            .checked_add(storage_geometry::retained_nodes::<usize, usize>(
                self.key_width_counts.len(),
            )?)
    }

    /// Current persistent-tree backing, excluding nested keys/values.
    /// Includes an allocated empty root and the derived key-width index.
    pub fn construction_storage_bytes(entries: usize) -> Option<usize> {
        storage_geometry::retained_nodes::<AspectKey, ContractValidatedAspectValue>(entries)?
            .checked_add(storage_geometry::retained_nodes::<usize, usize>(entries)?)
    }

    /// Peak incremental construction backing above independently retained input.
    /// Quotes this actual entry count and key width, including both empty roots.
    pub fn construction_scratch_bytes(entries: usize, maximum_key_width: usize) -> Option<usize> {
        Self::empty_edit_storage_bytes(entries.checked_mul(2)?, entries, maximum_key_width)
    }

    pub(crate) fn maximum_key_width(&self) -> usize {
        self.key_width_counts
            .get_max()
            .map_or(0, |(width, _)| *width)
    }

    /// Conservative copy-path scratch for this actual ordered edit batch.
    /// Extra metadata covers remove-then-insert and the width index's own edits.
    pub(crate) fn edit_storage_bytes(
        &self,
        operations: usize,
        additional_entries: usize,
        new_key_width: usize,
    ) -> Option<usize> {
        let entries = self.len().checked_add(additional_entries)?;
        let widths = self
            .key_width_counts
            .len()
            .checked_add(additional_entries)?;
        let key_bytes = self
            .maximum_key_width()
            .max(new_key_width)
            .checked_mul(64)?;
        storage_geometry::edit_paths::<AspectKey, ContractValidatedAspectValue>(
            entries, operations, key_bytes,
        )?
        .checked_add(storage_geometry::edit_paths::<usize, usize>(
            widths, operations, 0,
        )?)
    }

    pub(crate) fn empty_edit_storage_bytes(
        operations: usize,
        entries: usize,
        key_width: usize,
    ) -> Option<usize> {
        Self::construction_storage_bytes(0)?
            .checked_add(storage_geometry::edit_paths::<
                AspectKey,
                ContractValidatedAspectValue,
            >(
                entries, operations, key_width.checked_mul(64)?
            )?)?
            .checked_add(storage_geometry::edit_paths::<usize, usize>(
                entries, operations, 0,
            )?)
    }
}

#[cfg(test)]
mod tests;
