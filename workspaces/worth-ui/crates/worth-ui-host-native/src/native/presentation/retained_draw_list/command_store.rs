use std::collections::{HashMap, HashSet};

use worth_ui_host_contract::{UiMountedPaintCommand, UiMountedPaintCommandIdentity};

/// The ordinary retained-command owner deliberately has no `Clone` surface.
/// Complete replacement is reconstruction work; delta code may only address
/// exact identities through this bounded store.
pub(super) struct UiNativeRetainedCommandStore {
    by_identity: HashMap<UiMountedPaintCommandIdentity, UiMountedPaintCommand>,
    by_instance: HashMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        HashSet<UiMountedPaintCommandIdentity>,
    >,
}

impl UiNativeRetainedCommandStore {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            by_identity: HashMap::with_capacity(capacity),
            by_instance: HashMap::with_capacity(capacity),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.by_identity.len()
    }

    pub(super) fn contains(&self, identity: &UiMountedPaintCommandIdentity) -> bool {
        self.by_identity.contains_key(identity)
    }

    pub(super) fn get(
        &self,
        identity: &UiMountedPaintCommandIdentity,
    ) -> Option<&UiMountedPaintCommand> {
        self.by_identity.get(identity)
    }

    pub(super) fn insert(
        &mut self,
        identity: UiMountedPaintCommandIdentity,
        command: UiMountedPaintCommand,
    ) -> Option<UiMountedPaintCommand> {
        let previous = self.by_identity.insert(identity, command);
        if previous.is_none() {
            self.by_instance
                .entry(identity.mounted_instance())
                .or_default()
                .insert(identity);
        }
        previous
    }

    pub(super) fn remove(
        &mut self,
        identity: &UiMountedPaintCommandIdentity,
    ) -> Option<UiMountedPaintCommand> {
        let removed = self.by_identity.remove(identity)?;
        let instance = identity.mounted_instance();
        if let Some(identities) = self.by_instance.get_mut(&instance) {
            identities.remove(identity);
            if identities.is_empty() {
                self.by_instance.remove(&instance);
            }
        }
        Some(removed)
    }

    pub(super) fn identities_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.by_instance
            .get(&instance)
            .into_iter()
            .flat_map(|identities| identities.iter().copied())
    }

    pub(super) fn as_map(&self) -> &HashMap<UiMountedPaintCommandIdentity, UiMountedPaintCommand> {
        &self.by_identity
    }
}
