use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use super::super::appearance::UiMountedAppearanceSidecar;

const MEMBERSHIP_CAPACITY: usize = 4_096;

#[derive(Clone)]
pub(super) struct UiMountedAppearanceStateMembers {
    map: HashMap<UiMountedAppearanceStateKey, UiMountedAppearanceStateMembership>,
    pending_keys: Vec<UiMountedAppearanceStateKey>,
}

#[derive(Clone)]
pub(super) struct UiMountedAppearanceStateEntry {
    pub(super) key: UiMountedAppearanceStateKey,
    pub(super) context: crate::runtime::appearance::UiAppearanceAttemptContext,
    pub(super) projection: crate::runtime::appearance::UiAppearanceProjection,
    pub(super) sidecar: UiMountedAppearanceSidecar,
}

#[derive(Clone)]
pub(super) enum UiMountedAppearanceStateMembership {
    Retained(UiMountedAppearanceStateEntry),
    Reserved,
    Staged {
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
        predecessor: Option<UiMountedAppearanceStateEntry>,
    },
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct UiMountedAppearanceLocalNodeKey {
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(super) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) incarnation: worth_ui_host_contract::UiMountIncarnation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiMountedAppearanceStateKey {
    pub(super) session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(super) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) local_node: UiMountedAppearanceLocalNodeKey,
}

impl Hash for UiMountedAppearanceStateKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.session.as_u64().hash(state);
        self.generation.hash(state);
        self.local_node.hash(state);
    }
}

impl UiMountedAppearanceStateKey {
    pub(super) fn local_node(&self) -> &UiMountedAppearanceLocalNodeKey {
        &self.local_node
    }
}

impl Default for UiMountedAppearanceStateMembers {
    fn default() -> Self {
        Self::new()
    }
}

impl UiMountedAppearanceStateMembers {
    pub(super) fn new() -> Self {
        Self {
            map: HashMap::with_capacity(MEMBERSHIP_CAPACITY),
            pending_keys: Vec::with_capacity(MEMBERSHIP_CAPACITY),
        }
    }

    pub(super) fn inherit_from(&mut self, predecessor: Option<&Self>) {
        let mut map = HashMap::with_capacity(MEMBERSHIP_CAPACITY);
        if let Some(state) = predecessor {
            for (key, membership) in &state.map {
                if let UiMountedAppearanceStateMembership::Retained(entry) = membership {
                    map.insert(
                        key.clone(),
                        UiMountedAppearanceStateMembership::Retained(entry.clone()),
                    );
                }
            }
        }
        self.map = map;
        self.pending_keys.clear();
    }

    pub(super) fn prune_to_current_nodes(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        nodes: &[super::super::UiMountedAppearanceNodeInputContext],
    ) {
        let mut current_nodes = HashSet::with_capacity(nodes.len().min(MEMBERSHIP_CAPACITY));
        current_nodes.extend(nodes.iter().map(local_node_key));
        self.map.retain(|key, _| {
            key.session == session
                && key.generation == *generation
                && current_nodes.contains(key.local_node())
        });
        self.pending_keys.retain(|key| self.map.contains_key(key));
    }

    pub(super) fn reserve(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
        capacity: usize,
    ) -> Result<(), super::UiAppearanceStateCapacityExceeded> {
        let key = state_key(context);
        if self.map.contains_key(&key) {
            return Ok(());
        }
        if self.map.len() >= capacity {
            return Err(super::UiAppearanceStateCapacityExceeded::new(capacity));
        }
        self.map
            .insert(key.clone(), UiMountedAppearanceStateMembership::Reserved);
        self.pending_keys.push(key);
        Ok(())
    }

    pub(super) fn stage(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
        capacity: usize,
    ) -> Result<(), super::UiAppearanceStateCapacityExceeded> {
        self.reserve(attempt.context(), capacity)?;
        let key = state_key(attempt.context());
        let predecessor = match self.map.remove(&key) {
            Some(UiMountedAppearanceStateMembership::Retained(entry)) => {
                self.pending_keys.push(key.clone());
                Some(entry)
            }
            Some(UiMountedAppearanceStateMembership::Reserved) => None,
            Some(UiMountedAppearanceStateMembership::Staged { predecessor, .. }) => predecessor,
            None => unreachable!("reserved appearance key must be present before staging"),
        };
        // A duplicate stage replaces the older attempt for the same key. It
        // retains one membership slot and therefore cannot create duplicate work.
        self.map.insert(
            key,
            UiMountedAppearanceStateMembership::Staged {
                attempt,
                predecessor,
            },
        );
        Ok(())
    }

    pub(super) fn take_pending_keys_for_lowering(&mut self) -> Vec<UiMountedAppearanceStateKey> {
        let mut pending = std::mem::take(&mut self.pending_keys);
        pending.sort_by(|left, right| left.local_node().cmp(right.local_node()));
        pending
    }

    pub(super) fn remove(
        &mut self,
        key: &UiMountedAppearanceStateKey,
    ) -> Option<UiMountedAppearanceStateMembership> {
        self.map.remove(key)
    }

    pub(super) fn insert_retained(
        &mut self,
        entry: UiMountedAppearanceStateEntry,
    ) -> Option<UiMountedAppearanceStateMembership> {
        self.map.insert(
            entry.key.clone(),
            UiMountedAppearanceStateMembership::Retained(entry),
        )
    }

    pub(super) fn retained_entry_mut(
        &mut self,
        key: &UiMountedAppearanceStateKey,
    ) -> Option<&mut UiMountedAppearanceStateEntry> {
        match self.map.get_mut(key) {
            Some(UiMountedAppearanceStateMembership::Retained(entry)) => Some(entry),
            Some(UiMountedAppearanceStateMembership::Reserved)
            | Some(UiMountedAppearanceStateMembership::Staged { .. })
            | None => None,
        }
    }

    pub(super) fn retained_keys_for_reconstruction(&self) -> Vec<UiMountedAppearanceStateKey> {
        let mut keys = self
            .map
            .iter()
            .filter_map(|(key, membership)| match membership {
                UiMountedAppearanceStateMembership::Retained(_) => Some(key.clone()),
                UiMountedAppearanceStateMembership::Reserved
                | UiMountedAppearanceStateMembership::Staged { .. } => None,
            })
            .collect::<Vec<_>>();
        keys.sort_by(|left, right| left.local_node().cmp(right.local_node()));
        keys
    }

    #[cfg(test)]
    pub(super) fn retained_entry(
        &self,
        key: &UiMountedAppearanceStateKey,
    ) -> Option<&UiMountedAppearanceStateEntry> {
        match self.map.get(key) {
            Some(UiMountedAppearanceStateMembership::Retained(entry)) => Some(entry),
            Some(UiMountedAppearanceStateMembership::Reserved)
            | Some(UiMountedAppearanceStateMembership::Staged { .. })
            | None => None,
        }
    }

    #[cfg(test)]
    pub(super) fn membership_counts(&self) -> (usize, usize, usize) {
        self.map.values().fold((0, 0, 0), |counts, membership| {
            let (retained, reserved, staged) = counts;
            match membership {
                UiMountedAppearanceStateMembership::Retained(_) => (retained + 1, reserved, staged),
                UiMountedAppearanceStateMembership::Reserved => (retained, reserved + 1, staged),
                UiMountedAppearanceStateMembership::Staged { .. } => {
                    (retained, reserved, staged + 1)
                }
            }
        })
    }
}

pub(super) fn state_key(
    context: &crate::runtime::appearance::UiAppearanceAttemptContext,
) -> UiMountedAppearanceStateKey {
    let generation = context.generation().clone();
    UiMountedAppearanceStateKey {
        session: context.target().session(),
        generation,
        local_node: UiMountedAppearanceLocalNodeKey {
            surface: context.semantic_surface(),
            graph_node: context.graph_node(),
            mounted_instance: context.mounted_instance(),
            incarnation: context.incarnation(),
        },
    }
}

pub(super) fn local_node_key(
    node: &super::super::UiMountedAppearanceNodeInputContext,
) -> UiMountedAppearanceLocalNodeKey {
    UiMountedAppearanceLocalNodeKey {
        surface: node.semantic_surface,
        graph_node: node.graph_node,
        mounted_instance: node.mounted_instance,
        incarnation: node.incarnation,
    }
}
