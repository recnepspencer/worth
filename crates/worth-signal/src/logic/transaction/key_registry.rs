use std::num::NonZeroU32;

use crate::data::output::{ComputationFamily, ComputationKey, StructuralMemoKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct RuntimeStringId(NonZeroU32);

impl RuntimeStringId {
    pub(super) fn from_index(index: usize) -> Self {
        Self(NonZeroU32::new((index + 1) as u32).expect("runtime string ids are one-based"))
    }

    pub(super) fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeKeyRegistry {
    pub(super) families: crate::data::persistent_vector::PersistentVector<ComputationFamily>,
    pub(super) family_lookup:
        crate::data::persistent_ord_map::PersistentOrdMap<ComputationFamily, RuntimeStringId>,
    pub(super) keys: crate::data::persistent_vector::PersistentVector<ComputationKey>,
    pub(super) key_lookup:
        crate::data::persistent_ord_map::PersistentOrdMap<ComputationKey, RuntimeStringId>,
    pub(super) memo_keys: crate::data::persistent_vector::PersistentVector<StructuralMemoKey>,
    pub(super) memo_key_lookup:
        crate::data::persistent_ord_map::PersistentOrdMap<StructuralMemoKey, RuntimeStringId>,
}

impl RuntimeKeyRegistry {
    pub(super) fn fork_persistent(&mut self) -> Self {
        Self {
            families: self.families.fork_persistent(),
            family_lookup: self.family_lookup.fork_persistent(),
            keys: self.keys.fork_persistent(),
            key_lookup: self.key_lookup.fork_persistent(),
            memo_keys: self.memo_keys.fork_persistent(),
            memo_key_lookup: self.memo_key_lookup.fork_persistent(),
        }
    }

    pub(super) fn intern_family(&mut self, family: &ComputationFamily) -> RuntimeStringId {
        if let Some(id) = self.family_lookup.get(family).copied() {
            return id;
        }
        let owned = family.clone();
        let id = RuntimeStringId::from_index(self.families.len());
        self.families.push_back(owned.clone());
        self.family_lookup.insert(owned, id);
        id
    }

    pub(super) fn intern_key(&mut self, key: &ComputationKey) -> RuntimeStringId {
        if let Some(id) = self.key_lookup.get(key).copied() {
            return id;
        }
        let owned = key.clone();
        let id = RuntimeStringId::from_index(self.keys.len());
        self.keys.push_back(owned.clone());
        self.key_lookup.insert(owned, id);
        id
    }

    pub(super) fn intern_memo_key(&mut self, memo_key: &StructuralMemoKey) -> RuntimeStringId {
        if let Some(id) = self.memo_key_lookup.get(memo_key).copied() {
            return id;
        }
        let owned = memo_key.clone();
        let id = RuntimeStringId::from_index(self.memo_keys.len());
        self.memo_keys.push_back(owned.clone());
        self.memo_key_lookup.insert(owned, id);
        id
    }

    pub(super) fn family(&self, id: RuntimeStringId) -> &ComputationFamily {
        &self.families[id.index()]
    }

    pub(super) fn memo_key(&self, id: RuntimeStringId) -> &StructuralMemoKey {
        &self.memo_keys[id.index()]
    }

    #[cfg(test)]
    pub(super) fn fork_storage_identity(&self) -> Self {
        Self {
            families: self.families.clone(),
            family_lookup: self.family_lookup.fork_storage_identity(),
            keys: self.keys.clone(),
            key_lookup: self.key_lookup.fork_storage_identity(),
            memo_keys: self.memo_keys.clone(),
            memo_key_lookup: self.memo_key_lookup.fork_storage_identity(),
        }
    }

    #[cfg(test)]
    pub(super) fn shares_storage_with(&self, other: &Self) -> bool {
        self.families.shares_storage_with(&other.families)
            && self.family_lookup.ptr_eq(&other.family_lookup)
            && self.keys.shares_storage_with(&other.keys)
            && self.key_lookup.ptr_eq(&other.key_lookup)
            && self.memo_keys.shares_storage_with(&other.memo_keys)
            && self.memo_key_lookup.ptr_eq(&other.memo_key_lookup)
    }
}
