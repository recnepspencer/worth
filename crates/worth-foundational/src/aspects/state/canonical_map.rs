use im::OrdMap;

use super::AuthoritativeStateAdmissionDenial;
use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct CanonicalAspectStateMap {
    pub(super) entries: OrdMap<AspectKey, ContractValidatedAspectValue>,
    #[serde(skip)]
    allocation_bytes: u128,
}

impl CanonicalAspectStateMap {
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
        Self {
            entries,
            allocation_bytes,
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
        }
        self.entries.remove(key);
    }
    pub(crate) fn insert(&mut self, key: AspectKey, value: ContractValidatedAspectValue) {
        self.remove(&key);
        self.allocation_bytes += entry_bytes(&key, &value);
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
