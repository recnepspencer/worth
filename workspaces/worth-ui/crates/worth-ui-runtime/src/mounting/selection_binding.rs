use crate::runtime::selection::{
    UiSelectionAppearanceChange, UiSelectionOwnerIdentity, UiSelectionOwnerIncarnation,
    UiSelectionProjectionMapping, UiSelectionStableKey,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_ui_host_contract::{UiMountIncarnation, UiMountedInstanceIdentity};

type Owner = (UiSelectionOwnerIdentity, UiSelectionOwnerIncarnation);

/// Mounted item provenance, never selection membership or interaction state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiMountedSelectionItem {
    pub(super) owner_instance: UiMountedInstanceIdentity,
    pub(super) item_incarnation: UiMountIncarnation,
    pub(super) option: worth_ui_query_binding::UiProjectionOptionReference,
    pub(super) mapping: UiSelectionProjectionMapping,
}

#[derive(Default)]
pub(super) struct UiMountedSelectionBindings {
    items: BTreeMap<UiMountedInstanceIdentity, UiMountedSelectionItem>,
    owners: BTreeMap<Owner, BTreeMap<UiSelectionStableKey, BTreeSet<UiMountedInstanceIdentity>>>,
    mounts: BTreeMap<UiMountedInstanceIdentity, BTreeSet<UiMountedInstanceIdentity>>,
    surfaces: BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        BTreeSet<UiMountedInstanceIdentity>,
    >,
    slots: BTreeMap<
        worth_ui_query_binding::UiProjectionInputSlot,
        BTreeSet<UiMountedInstanceIdentity>,
    >,
    changed: BTreeSet<UiMountedInstanceIdentity>,
}

impl UiMountedSelectionBindings {
    pub(super) fn get(&self, item: UiMountedInstanceIdentity) -> Option<&UiMountedSelectionItem> {
        self.items.get(&item)
    }

    pub(super) fn owner_conflicts(&self, mapping: UiSelectionProjectionMapping) -> bool {
        let first = (mapping.owner, UiSelectionOwnerIncarnation::new(1).unwrap());
        let last = (
            mapping.owner,
            UiSelectionOwnerIncarnation::new(u64::MAX).unwrap(),
        );
        self.owners
            .range(first..=last)
            .any(|((_, incarnation), _)| *incarnation != mapping.incarnation)
    }

    pub(super) fn retire_surface(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) {
        let items = self.surfaces.get(&surface).cloned().unwrap_or_default();
        for item in items {
            self.remove(item);
        }
    }

    pub(super) fn insert(
        &mut self,
        item: UiMountedInstanceIdentity,
        binding: UiMountedSelectionItem,
    ) {
        if self.items.get(&item) == Some(&binding) {
            return;
        }
        self.remove(item);
        self.owners
            .entry((binding.mapping.owner, binding.mapping.incarnation))
            .or_default()
            .entry(binding.mapping.key)
            .or_default()
            .insert(item);
        self.mounts
            .entry(binding.owner_instance)
            .or_default()
            .insert(item);
        self.surfaces
            .entry(binding.mapping.owner.semantic_surface())
            .or_default()
            .insert(item);
        self.slots
            .entry(binding.option.owner_revision().slot())
            .or_default()
            .insert(item);
        self.items.insert(item, binding);
        self.changed.insert(item);
    }

    pub(super) fn remove(&mut self, item: UiMountedInstanceIdentity) {
        let Some(binding) = self.items.remove(&item) else {
            return;
        };
        let owner = (binding.mapping.owner, binding.mapping.incarnation);
        if let Some(keys) = self.owners.get_mut(&owner) {
            if let Some(items) = keys.get_mut(&binding.mapping.key) {
                items.remove(&item);
                if items.is_empty() {
                    keys.remove(&binding.mapping.key);
                }
            }
            if keys.is_empty() {
                self.owners.remove(&owner);
            }
        }
        remove_member(&mut self.mounts, binding.owner_instance, item);
        remove_member(
            &mut self.surfaces,
            binding.mapping.owner.semantic_surface(),
            item,
        );
        remove_member(
            &mut self.slots,
            binding.option.owner_revision().slot(),
            item,
        );
        self.changed.insert(item);
    }

    pub(super) fn retire_mount(&mut self, instance: UiMountedInstanceIdentity) {
        let items = self.mounts.get(&instance).cloned().unwrap_or_default();
        for item in items {
            self.remove(item);
        }
        self.remove(instance);
    }

    pub(super) fn for_slot(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Vec<UiMountedInstanceIdentity> {
        self.slots
            .get(&slot)
            .map(|items| items.iter().copied().collect())
            .unwrap_or_default()
    }

    pub(super) fn slot_revision(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<&worth_ui_query_binding::UiProjectionInputRevision> {
        self.items
            .get(self.slots.get(&slot)?.first()?)
            .map(|item| item.option.owner_revision())
    }

    pub(super) fn all_items(&self) -> Vec<UiMountedInstanceIdentity> {
        self.items.keys().copied().collect()
    }

    pub(super) fn clear(&mut self) {
        self.changed.extend(self.items.keys().copied());
        self.items.clear();
        self.owners.clear();
        self.mounts.clear();
        self.slots.clear();
        self.surfaces.clear();
    }

    pub(super) fn take_changed(&mut self) -> Vec<UiMountedInstanceIdentity> {
        std::mem::take(&mut self.changed).into_iter().collect()
    }

    pub(super) fn affected(
        &self,
        changes: &[UiSelectionAppearanceChange],
    ) -> Vec<UiMountedInstanceIdentity> {
        let mut affected = BTreeSet::new();
        for change in changes {
            match change {
                UiSelectionAppearanceChange::Keys {
                    owner,
                    incarnation,
                    keys,
                } => {
                    if let Some(bound) = self.owners.get(&(*owner, *incarnation)) {
                        for key in keys {
                            if let Some(items) = bound.get(key) {
                                affected.extend(items);
                            }
                        }
                    }
                }
                UiSelectionAppearanceChange::Owner {
                    owner,
                    previous,
                    current,
                } => {
                    for incarnation in [previous, current].into_iter().flatten() {
                        if let Some(keys) = self.owners.get(&(*owner, *incarnation)) {
                            for items in keys.values() {
                                affected.extend(items);
                            }
                        }
                    }
                }
            }
        }
        affected.into_iter().collect()
    }
}

fn remove_member<K: Ord>(
    index: &mut BTreeMap<K, BTreeSet<UiMountedInstanceIdentity>>,
    key: K,
    item: UiMountedInstanceIdentity,
) {
    if let Some(items) = index.get_mut(&key) {
        items.remove(&item);
        if items.is_empty() {
            index.remove(&key);
        }
    }
}
