use std::collections::{BTreeMap, BTreeSet};

mod declared_activation;
mod inspection;
mod lifecycle;
mod record;

use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentOrdSet};
pub(super) use record::UiSelectionOwnerRecord;
mod synchronization;
pub(super) use record::validate_catalog;

/// Sole owner of selection, range anchor, and selection cursor state. Query
/// contributes opaque stable row correlation only.
#[derive(Clone)]
pub(crate) struct UiSelectionRuntimeState {
    persistence: crate::runtime::UiServiceStatePersistencePosture,
    policy: crate::declaration::UiSelectionPolicy,
    pub(super) owners: UiPersistentOrdMap<super::UiSelectionOwnerIdentity, UiSelectionOwnerRecord>,
    pub(super) revision: u64,
    requests: u64,
    candidates_visited: u64,
    catalog_keys_reconciled: u64,
    pub(super) mounted_owners: UiPersistentOrdMap<
        (
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            crate::graph::UiGraphNodeIdentity,
            super::UiSelectionOwnerIncarnation,
        ),
        UiPersistentOrdSet<super::UiSelectionOwnerIdentity>,
    >,
    family_owners: BTreeMap<
        crate::runtime::UiApplicationItemKeyFamily,
        BTreeSet<super::UiSelectionOwnerIdentity>,
    >,
    last_drop: Option<super::UiSelectionDropInspectionRecord>,
}

impl UiSelectionRuntimeState {
    pub(crate) const fn new_session_restore_candidate() -> Self {
        Self::new_session_restore_candidate_with_policy(
            crate::declaration::UiSelectionPolicy::single(),
        )
    }

    pub(crate) const fn new_session_restore_candidate_with_policy(
        policy: crate::declaration::UiSelectionPolicy,
    ) -> Self {
        Self {
            persistence: crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
            policy,
            owners: UiPersistentOrdMap::new(),
            revision: 0,
            requests: 0,
            candidates_visited: 0,
            catalog_keys_reconciled: 0,
            mounted_owners: UiPersistentOrdMap::new(),
            family_owners: BTreeMap::new(),
            last_drop: None,
        }
    }

    pub(crate) fn apply_policy(&mut self, policy: crate::declaration::UiSelectionPolicy) {
        self.policy = policy;
    }

    pub(crate) const fn default_owner_policy(&self) -> super::UiSelectionPolicy {
        match self.policy.mode() {
            crate::declaration::UiSelectionMode::Single => super::UiSelectionPolicy::Single,
            crate::declaration::UiSelectionMode::Multiple => super::UiSelectionPolicy::Multiple,
            crate::declaration::UiSelectionMode::Range => {
                super::UiSelectionPolicy::MultipleWithRange
            }
        }
    }

    pub(crate) fn apply(
        &mut self,
        owner: super::UiSelectionOwnerIdentity,
        incarnation: super::UiSelectionOwnerIncarnation,
        request: super::UiSelectionRequest,
    ) -> Result<super::UiSelectionDelta, super::UiSelectionRequestDenial> {
        let mut record = self
            .owners
            .get(&owner)
            .cloned()
            .ok_or(super::UiSelectionRequestDenial::UnknownOwner)?;
        if record.incarnation != incarnation {
            return Err(super::UiSelectionRequestDenial::StaleOwnerIncarnation);
        }
        if !record.catalog_available {
            return Err(super::UiSelectionRequestDenial::CatalogUnavailable);
        }
        let visited = super::reducer::validate_request(&record, request)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(super::UiSelectionRequestDenial::RevisionExhausted)?;
        let requests = self
            .requests
            .checked_add(1)
            .ok_or(super::UiSelectionRequestDenial::CounterOverflow)?;
        let candidates_visited = self
            .candidates_visited
            .checked_add(u64::from(visited))
            .ok_or(super::UiSelectionRequestDenial::CounterOverflow)?;
        let previous_positions = record.positions();
        let mutation = super::reducer::apply_request(&mut record, request)?;
        self.revision = revision;
        self.requests = requests;
        self.candidates_visited = candidates_visited;
        let delta = super::UiSelectionDelta::new(
            mutation.added,
            mutation.removed,
            record.selected.len(),
            visited,
            revision,
            super::UiSelectionPositionChanges::new(previous_positions, record.positions()),
        );
        record.revision = revision;
        self.owners.insert(owner, record);
        self.record_drop(
            owner,
            super::UiSelectionDropInspectionReason::Interaction,
            &delta,
        );
        Ok(delta)
    }

    #[cfg(test)]
    pub(crate) fn selected(
        &self,
        owner: super::UiSelectionOwnerIdentity,
    ) -> Option<&UiPersistentOrdSet<super::UiSelectionStableKey>> {
        self.owners.get(&owner).map(|record| &record.selected)
    }

    pub(crate) const fn selection_keys_visited(&self) -> u64 {
        self.candidates_visited
    }

    pub(crate) fn compact_posture_for(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        incarnation: super::UiSelectionOwnerIncarnation,
    ) -> Option<(usize, u64)> {
        let owners = self
            .mounted_owners
            .get(&(surface, graph_node, incarnation))?;
        if owners.len() != 1 {
            return None;
        }
        let record = self.owners.get(owners.iter().next()?)?;
        Some((record.selected.len(), self.revision))
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_for_certification(
        &self,
    ) -> (
        usize,
        usize,
        usize,
        u64,
        u64,
        u64,
        u64,
        Box<[core::num::NonZeroU64]>,
    ) {
        (
            self.owners.len(),
            self.owners
                .iter()
                .map(|(_, owner)| owner)
                .filter(|owner| owner.catalog_available)
                .count(),
            self.owners
                .iter()
                .map(|(_, owner)| owner)
                .map(|owner| owner.selected.len())
                .sum(),
            self.revision,
            self.requests,
            self.candidates_visited,
            self.catalog_keys_reconciled,
            self.owners
                .iter()
                .map(|(_, owner)| owner)
                .flat_map(|owner| owner.selected.iter().copied())
                .map(super::UiSelectionStableKey::application_value)
                .collect(),
        )
    }

    fn index_owner(
        &mut self,
        owner: super::UiSelectionOwnerIdentity,
        incarnation: super::UiSelectionOwnerIncarnation,
    ) {
        let mounted_key = (owner.semantic_surface(), owner.graph_node(), incarnation);
        let mut owners = self
            .mounted_owners
            .get(&mounted_key)
            .cloned()
            .unwrap_or_default();
        owners.insert(owner);
        self.mounted_owners.insert(mounted_key, owners);
        self.family_owners
            .entry(owner.key_family())
            .or_default()
            .insert(owner);
    }

    fn unindex_owner(
        &mut self,
        owner: super::UiSelectionOwnerIdentity,
        incarnation: super::UiSelectionOwnerIncarnation,
    ) {
        let mounted_key = (owner.semantic_surface(), owner.graph_node(), incarnation);
        let remove_mounted =
            if let Some(mut owners) = self.mounted_owners.get(&mounted_key).cloned() {
                owners.remove(&owner);
                let empty = owners.is_empty();
                if !empty {
                    self.mounted_owners.insert(mounted_key, owners);
                }
                empty
            } else {
                false
            };
        if remove_mounted {
            self.mounted_owners.remove(&mounted_key);
        }
        let family = owner.key_family();
        let remove_family = if let Some(owners) = self.family_owners.get_mut(&family) {
            owners.remove(&owner);
            owners.is_empty()
        } else {
            false
        };
        if remove_family {
            self.family_owners.remove(&family);
        }
    }
}
