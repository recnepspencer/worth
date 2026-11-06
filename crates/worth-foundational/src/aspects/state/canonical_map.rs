use std::collections::BTreeMap;

use super::AuthoritativeStateAdmissionDenial;
use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalAspectStateMap {
    entries: BTreeMap<AspectKey, ContractValidatedAspectValue>,
}

impl CanonicalAspectStateMap {
    pub(crate) fn from_validated_entries(
        entries: impl IntoIterator<Item = ContractValidatedAspectValue>,
    ) -> Result<Self, AuthoritativeStateAdmissionDenial> {
        let mut canonical_entries = BTreeMap::new();
        for entry in entries {
            let key = entry.key().clone();
            if canonical_entries.insert(key.clone(), entry).is_some() {
                return Err(AuthoritativeStateAdmissionDenial::DuplicateAspectKey(key));
            }
        }

        Ok(Self {
            entries: canonical_entries,
        })
    }

    pub(crate) fn from_canonical_entries(
        entries: BTreeMap<AspectKey, ContractValidatedAspectValue>,
    ) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&AspectKey, &ContractValidatedAspectValue)> {
        self.entries.iter()
    }

    pub fn get(&self, key: &AspectKey) -> Option<&ContractValidatedAspectValue> {
        self.entries.get(key)
    }

    pub(crate) fn cloned_entries(&self) -> BTreeMap<AspectKey, ContractValidatedAspectValue> {
        self.entries.clone()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Owner-accounted bytes allocated by the canonical map and every nested
    /// key, contract, and value payload. Allocator bookkeeping is excluded.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.entries.iter().fold(0_usize, |bytes, (key, value)| {
            bytes
                .saturating_add(std::mem::size_of::<(AspectKey, ContractValidatedAspectValue)>())
                .saturating_add(key.owned_allocation_capacity_bytes())
                .saturating_add(value.owned_allocation_capacity_bytes())
        })
    }
}
