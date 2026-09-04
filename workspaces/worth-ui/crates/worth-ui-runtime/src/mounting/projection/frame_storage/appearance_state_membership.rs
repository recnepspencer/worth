use super::super::appearance::UiMountedAppearanceSidecar;
use super::appearance_state_membership_work::UiMountedAppearanceMembershipWork;
use crate::runtime::persistent_index::UiPersistentOrdMap;

type Mutation<T> = (
    Result<T, super::UiMountedAppearanceStateMutationDenial>,
    UiMountedAppearanceMembershipWork,
);

#[derive(Clone)]
pub(super) struct UiMountedAppearanceStateMembers {
    primary:
        UiPersistentOrdMap<UiMountedAppearanceLocalNodeKey, UiMountedAppearanceStateMembership>,
    reverse: UiPersistentOrdMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        UiMountedAppearanceLocalNodeKey,
    >,
    pending_keys: Vec<UiMountedAppearanceLocalNodeKey>,
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

impl std::hash::Hash for UiMountedAppearanceStateKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
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
            primary: UiPersistentOrdMap::default(),
            reverse: UiPersistentOrdMap::default(),
            pending_keys: Vec::new(),
        }
    }

    pub(super) fn fork(&self) -> Self {
        Self {
            primary: self.primary.clone(),
            reverse: self.reverse.clone(),
            pending_keys: Vec::new(),
        }
    }

    pub(super) fn clear_for_epoch(&mut self) -> (usize, UiMountedAppearanceMembershipWork) {
        let retired = self.primary.len();
        let mut work = UiMountedAppearanceMembershipWork::default();
        // Dropping the two persistent roots is logically constant-time, while
        // releasing their bounded nodes is charged to lifecycle retirement.
        work.add_traversal(retired);
        self.primary = UiPersistentOrdMap::default();
        self.reverse = UiPersistentOrdMap::default();
        self.pending_keys.clear();
        (retired, work)
    }

    pub(super) fn retire_instances(
        &mut self,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> (usize, UiMountedAppearanceMembershipWork) {
        let mut retired = 0;
        let mut work = UiMountedAppearanceMembershipWork::default();
        for instance in instances {
            let (local_key, reverse_probes) = self.reverse.get_with_probes(instance);
            work.add_lookup(reverse_probes);
            let Some(local_key) = local_key.cloned() else {
                continue;
            };
            if local_key.mounted_instance != *instance {
                continue;
            }
            let (membership, remove_work) = self.remove(&local_key);
            work.merge(remove_work);
            if membership.is_some() {
                retired += 1;
            }
        }
        (retired, work)
    }

    pub(super) fn reserve(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
        capacity: usize,
    ) -> Mutation<()> {
        let key = state_key(context).local_node;
        let mut work = UiMountedAppearanceMembershipWork::default();
        let (reverse_key, reverse_probes) = self.reverse.get_with_probes(&key.mounted_instance);
        work.add_lookup(reverse_probes);
        if reverse_key.is_some_and(|existing| existing != &key) {
            return (
                Err(super::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch),
                work,
            );
        }
        let (existing, primary_probes) = self.primary.get_with_probes(&key);
        work.add_lookup(primary_probes);
        if existing.is_some() {
            return if reverse_key.is_some() {
                (Ok(()), work)
            } else {
                (
                    Err(super::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch),
                    work,
                )
            };
        }
        if reverse_key.is_some() {
            return (
                Err(super::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch),
                work,
            );
        }
        if self.primary.len() >= capacity {
            return (
                Err(super::UiMountedAppearanceStateMutationDenial::Capacity(
                    super::UiAppearanceStateCapacityExceeded::new(capacity),
                )),
                work,
            );
        }
        let primary_work = self
            .primary
            .insert_with_work(key.clone(), UiMountedAppearanceStateMembership::Reserved);
        work.add_mutation(primary_work);
        let reverse_work = self
            .reverse
            .insert_with_work(key.mounted_instance, key.clone());
        work.add_mutation(reverse_work);
        self.pending_keys.push(key);
        (Ok(()), work)
    }

    pub(super) fn stage(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
        capacity: usize,
    ) -> Mutation<()> {
        let (reserved, mut work) = self.reserve(attempt.context(), capacity);
        if let Err(error) = reserved {
            return (Err(error), work);
        }
        let key = state_key(attempt.context()).local_node;
        let (membership, remove_work) = self.remove(&key);
        work.merge(remove_work);
        let predecessor = match membership {
            Some(UiMountedAppearanceStateMembership::Retained(entry)) => {
                self.pending_keys.push(key.clone());
                Some(entry)
            }
            Some(UiMountedAppearanceStateMembership::Reserved) => None,
            Some(UiMountedAppearanceStateMembership::Staged { predecessor, .. }) => predecessor,
            None => unreachable!("reserved appearance key must be present before staging"),
        };
        let (inserted, insert_work) = self.insert_membership(
            key,
            UiMountedAppearanceStateMembership::Staged {
                attempt,
                predecessor,
            },
        );
        work.merge(insert_work);
        (inserted.map(|_| ()), work)
    }

    pub(super) fn take_pending_keys_for_lowering(
        &mut self,
    ) -> Vec<UiMountedAppearanceLocalNodeKey> {
        let mut pending = std::mem::take(&mut self.pending_keys);
        pending.sort();
        pending
    }

    pub(super) fn remove(
        &mut self,
        key: &UiMountedAppearanceLocalNodeKey,
    ) -> (
        Option<UiMountedAppearanceStateMembership>,
        UiMountedAppearanceMembershipWork,
    ) {
        let mut work = UiMountedAppearanceMembershipWork::default();
        let (membership, primary_probes) = self.primary.get_with_probes(key);
        work.add_lookup(primary_probes);
        let Some(membership) = membership.cloned() else {
            return (None, work);
        };
        let (removed, primary_work) = self.primary.remove_with_work(key);
        debug_assert!(removed);
        work.add_mutation(primary_work);
        let (reverse_key, reverse_probes) = self.reverse.get_with_probes(&key.mounted_instance);
        work.add_lookup(reverse_probes);
        if reverse_key.is_some_and(|existing| existing == key) {
            let (removed, reverse_work) = self.reverse.remove_with_work(&key.mounted_instance);
            debug_assert!(removed);
            work.add_mutation(reverse_work);
        }
        (Some(membership), work)
    }

    pub(super) fn insert_retained(
        &mut self,
        entry: UiMountedAppearanceStateEntry,
    ) -> Mutation<Option<UiMountedAppearanceStateMembership>> {
        self.insert_membership(
            entry.key.local_node.clone(),
            UiMountedAppearanceStateMembership::Retained(entry),
        )
    }

    fn insert_membership(
        &mut self,
        key: UiMountedAppearanceLocalNodeKey,
        membership: UiMountedAppearanceStateMembership,
    ) -> Mutation<Option<UiMountedAppearanceStateMembership>> {
        let mut work = UiMountedAppearanceMembershipWork::default();
        let (reverse_key, reverse_probes) = self.reverse.get_with_probes(&key.mounted_instance);
        work.add_lookup(reverse_probes);
        if reverse_key.is_some_and(|existing| existing != &key) {
            return (
                Err(super::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch),
                work,
            );
        }
        let (existing, primary_probes) = self.primary.get_with_probes(&key);
        work.add_lookup(primary_probes);
        if reverse_key.is_none() && existing.is_some() {
            return (
                Err(super::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch),
                work,
            );
        }
        let old = existing.cloned();
        let primary_work = self.primary.insert_with_work(key.clone(), membership);
        work.add_mutation(primary_work);
        if reverse_key.is_none() {
            let reverse_work = self.reverse.insert_with_work(key.mounted_instance, key);
            work.add_mutation(reverse_work);
        }
        (Ok(old), work)
    }

    pub(super) fn retained_entry(
        &self,
        key: &UiMountedAppearanceStateKey,
    ) -> Option<&UiMountedAppearanceStateEntry> {
        match self.primary.get(key.local_node()) {
            Some(membership) => match membership {
                UiMountedAppearanceStateMembership::Retained(entry) => Some(entry),
                UiMountedAppearanceStateMembership::Reserved
                | UiMountedAppearanceStateMembership::Staged { .. } => None,
            },
            None => None,
        }
    }

    pub(super) fn retained_keys_for_reconstruction(
        &self,
    ) -> (
        Vec<UiMountedAppearanceStateKey>,
        UiMountedAppearanceMembershipWork,
    ) {
        let mut work = UiMountedAppearanceMembershipWork::default();
        work.add_traversal(self.primary.len());
        let mut keys = self
            .primary
            .iter()
            .filter_map(|(local_key, membership)| match membership {
                UiMountedAppearanceStateMembership::Retained(entry) => Some(entry.key.clone()),
                UiMountedAppearanceStateMembership::Reserved
                | UiMountedAppearanceStateMembership::Staged { .. } => {
                    let _ = local_key;
                    None
                }
            })
            .collect::<Vec<_>>();
        keys.sort_by(|left, right| left.local_node().cmp(right.local_node()));
        (keys, work)
    }

    #[cfg(test)]
    pub(super) fn membership_counts(&self) -> (usize, usize, usize) {
        self.primary
            .iter()
            .fold((0, 0, 0), |counts, (_, membership)| {
                let (retained, reserved, staged) = counts;
                match membership {
                    UiMountedAppearanceStateMembership::Retained(_) => {
                        (retained + 1, reserved, staged)
                    }
                    UiMountedAppearanceStateMembership::Reserved => {
                        (retained, reserved + 1, staged)
                    }
                    UiMountedAppearanceStateMembership::Staged { .. } => {
                        (retained, reserved, staged + 1)
                    }
                }
            })
    }

    #[cfg(test)]
    pub(super) fn roots_shared_with(&self, other: &Self) -> bool {
        self.primary.root_is_shared_with(&other.primary)
            && self.reverse.root_is_shared_with(&other.reverse)
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
