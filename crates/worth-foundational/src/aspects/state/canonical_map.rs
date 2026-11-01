use super::AuthoritativeStateAdmissionDenial;
use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalAspectStateMap {
    #[serde(serialize_with = "serialize_entries")]
    entries: Box<[ContractValidatedAspectValue]>,
}

impl CanonicalAspectStateMap {
    pub(crate) fn from_validated_entries(
        entries: impl IntoIterator<Item = ContractValidatedAspectValue>,
    ) -> Result<Self, AuthoritativeStateAdmissionDenial> {
        // Ordinary generic admission preserves its lazy first-duplicate refusal.
        // The owned patch application path does not enter this temporary map.
        let mut canonical_entries = BTreeMap::new();
        for entry in entries {
            let key = entry.key().clone();
            if canonical_entries.insert(key.clone(), entry).is_some() {
                return Err(AuthoritativeStateAdmissionDenial::DuplicateAspectKey(key));
            }
        }
        Ok(Self {
            entries: canonical_entries
                .into_values()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }

    pub(crate) fn from_canonical_entries(entries: Box<[ContractValidatedAspectValue]>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&AspectKey, &ContractValidatedAspectValue)> {
        self.entries.iter().map(|entry| (entry.key(), entry))
    }

    pub fn get(&self, key: &AspectKey) -> Option<&ContractValidatedAspectValue> {
        self.entries
            .binary_search_by(|entry| entry.key().cmp(key))
            .ok()
            .map(|index| &self.entries[index])
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Actual boxed backing and nested allocation capacities, excluding the
    /// inline wrapper and allocator bookkeeping.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.entries.iter().fold(
            self.entries
                .len()
                .saturating_mul(std::mem::size_of::<ContractValidatedAspectValue>()),
            |bytes, value| bytes.saturating_add(value.owned_allocation_capacity_bytes()),
        )
    }

    /// Descriptive backing bound only; nested contract/key/value clones are
    /// separately funded by the caller. Boxing may overlap the original Vec.
    pub fn construction_storage_bytes(entry_count: usize) -> Option<usize> {
        std::alloc::Layout::array::<ContractValidatedAspectValue>(entry_count)
            .ok()?
            .size()
            .checked_mul(2)
    }
}

fn serialize_entries<S: Serializer>(
    entries: &[ContractValidatedAspectValue],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(entries.len()))?;
    for entry in entries {
        map.serialize_entry(entry.key(), entry)?;
    }
    map.end()
}
