use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

#[derive(Clone, Default)]
pub(crate) struct UiMountedProjectionChanges {
    changed_instances:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedInstanceIdentity>,
    appearance_input_changed_instances:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedInstanceIdentity>,
    retired_instances:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedInstanceIdentity>,
    changed_surfaces:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiSemanticSurfaceIdentity>,
    removed_surfaces:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiSemanticSurfaceIdentity>,
    order_changed: bool,
    coalesced: u64,
    overflowed: bool,
}

#[derive(Clone)]
pub(crate) struct UiMountedProjectionChangeSnapshot {
    semantic_revision: u64,
    binding_revision: u64,
    observed: UiMountedProjectionChanges,
    applied: UiMountedProjectionChanges,
    remainder: UiMountedProjectionChanges,
}

impl UiMountedProjectionChanges {
    pub(crate) fn mark_changed_instance(&mut self, instance: UiMountedInstanceIdentity) {
        self.retired_instances.remove_with_work(&instance);
        if !self.changed_instances.insert(instance) {
            self.record_coalesced();
        }
    }

    pub(crate) fn mark_appearance_input_changed(&mut self, instance: UiMountedInstanceIdentity) {
        if !self.appearance_input_changed_instances.insert(instance) {
            self.record_coalesced();
        }
    }

    pub(crate) fn mark_retired_instance(&mut self, instance: UiMountedInstanceIdentity) {
        self.changed_instances.remove_with_work(&instance);
        self.appearance_input_changed_instances
            .remove_with_work(&instance);
        if !self.retired_instances.insert(instance) {
            self.record_coalesced();
        }
    }

    pub(crate) fn mark_order_changed(&mut self, order: &[UiMountedInstanceIdentity]) {
        if self.order_changed {
            self.record_coalesced();
        }
        self.order_changed = true;
        for instance in order {
            self.mark_changed_instance(*instance);
        }
    }

    pub(crate) fn mark_changed_surface(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.removed_surfaces.remove_with_work(&surface);
        if !self.changed_surfaces.insert(surface) {
            self.record_coalesced();
        }
    }

    pub(crate) fn mark_removed_surface(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.changed_surfaces.remove_with_work(&surface);
        if !self.removed_surfaces.insert(surface) {
            self.record_coalesced();
        }
    }

    pub(crate) fn snapshot(
        &self,
        semantic_revision: u64,
        binding_revision: u64,
    ) -> UiMountedProjectionChangeSnapshot {
        UiMountedProjectionChangeSnapshot {
            semantic_revision,
            binding_revision,
            observed: self.clone(),
            applied: self.clone(),
            remainder: UiMountedProjectionChanges::default(),
        }
    }

    fn apply(&mut self, applied: &Self) {
        for instance in applied.changed_instances.iter() {
            self.changed_instances.remove_with_work(instance);
        }
        for instance in applied.appearance_input_changed_instances.iter() {
            self.appearance_input_changed_instances
                .remove_with_work(instance);
        }
        for instance in applied.retired_instances.iter() {
            self.retired_instances.remove_with_work(instance);
        }
        for surface in applied.changed_surfaces.iter() {
            self.changed_surfaces.remove_with_work(surface);
        }
        for surface in applied.removed_surfaces.iter() {
            self.removed_surfaces.remove_with_work(surface);
        }
        if applied.order_changed {
            self.order_changed = false;
        }
        if self.is_empty() {
            self.coalesced = 0;
            self.overflowed = false;
        }
    }

    fn merge(&mut self, addition: &Self) {
        for instance in addition.changed_instances.iter().copied() {
            self.mark_changed_instance(instance);
        }
        for instance in addition.appearance_input_changed_instances.iter().copied() {
            if !self.appearance_input_changed_instances.insert(instance) {
                self.record_coalesced();
            }
        }
        for instance in addition.retired_instances.iter().copied() {
            self.mark_retired_instance(instance);
        }
        for surface in addition.changed_surfaces.iter().copied() {
            self.mark_changed_surface(surface);
        }
        for surface in addition.removed_surfaces.iter().copied() {
            self.mark_removed_surface(surface);
        }
        self.order_changed |= addition.order_changed;
        self.coalesced = self.coalesced.saturating_add(addition.coalesced);
        self.overflowed |= addition.overflowed;
    }

    fn is_empty(&self) -> bool {
        self.changed_instances.is_empty()
            && self.appearance_input_changed_instances.is_empty()
            && self.retired_instances.is_empty()
            && self.changed_surfaces.is_empty()
            && self.removed_surfaces.is_empty()
            && !self.order_changed
    }

    fn record_coalesced(&mut self) {
        match self.coalesced.checked_add(1) {
            Some(coalesced) => self.coalesced = coalesced,
            None => self.overflowed = true,
        }
    }
}

impl UiMountedProjectionChangeSnapshot {
    pub(crate) fn changed_instances(&self) -> impl Iterator<Item = UiMountedInstanceIdentity> + '_ {
        self.applied.changed_instances.iter().copied()
    }

    pub(crate) fn retired_instances(&self) -> impl Iterator<Item = UiMountedInstanceIdentity> + '_ {
        self.applied.retired_instances.iter().copied()
    }

    pub(crate) fn appearance_input_changed(&self, instance: UiMountedInstanceIdentity) -> bool {
        self.applied
            .appearance_input_changed_instances
            .contains_with_probes(&instance)
            .0
    }

    pub(crate) fn appearance_input_changed_instances(
        &self,
    ) -> impl Iterator<Item = UiMountedInstanceIdentity> + '_ {
        self.applied
            .appearance_input_changed_instances
            .iter()
            .copied()
    }

    pub(crate) fn changed_surfaces(&self) -> impl Iterator<Item = UiSemanticSurfaceIdentity> + '_ {
        self.applied.changed_surfaces.iter().copied()
    }

    pub(crate) fn removed_surfaces(&self) -> impl Iterator<Item = UiSemanticSurfaceIdentity> + '_ {
        self.applied.removed_surfaces.iter().copied()
    }

    pub(crate) fn affects_surface(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.applied
            .changed_surfaces
            .contains_with_probes(&surface)
            .0
            || self
                .applied
                .removed_surfaces
                .contains_with_probes(&surface)
                .0
    }

    pub(crate) fn order_changed(&self) -> bool {
        self.applied.order_changed
    }

    pub(crate) fn coalesced(&self) -> u64 {
        self.applied.coalesced
    }

    pub(crate) fn overflowed(&self) -> bool {
        self.applied.overflowed
    }

    pub(crate) fn for_reconciliation(
        &self,
        current_instances: &[UiMountedInstanceIdentity],
        reconciled_surfaces: &[UiSemanticSurfaceIdentity],
    ) -> Option<Self> {
        let current_changed = current_instances.iter().any(|instance| {
            self.observed
                .changed_instances
                .contains_with_probes(instance)
                .0
                || self
                    .observed
                    .retired_instances
                    .contains_with_probes(instance)
                    .0
        });
        if self.observed.order_changed || current_changed {
            return None;
        }
        let mut applied = UiMountedProjectionChanges::default();
        for surface in reconciled_surfaces {
            if self
                .observed
                .changed_surfaces
                .contains_with_probes(surface)
                .0
            {
                applied.mark_changed_surface(*surface);
            }
            if self
                .observed
                .removed_surfaces
                .contains_with_probes(surface)
                .0
            {
                applied.mark_removed_surface(*surface);
            }
        }
        let mut remainder = self.observed.clone();
        remainder.apply(&applied);
        Some(Self {
            semantic_revision: self.semantic_revision,
            binding_revision: self.binding_revision,
            observed: self.observed.clone(),
            applied,
            remainder,
        })
    }

    pub(crate) fn matches(&self, semantic_revision: u64, binding_revision: u64) -> bool {
        self.semantic_revision == semantic_revision && self.binding_revision == binding_revision
    }

    pub(crate) fn commit_into(&self, pending: &mut UiMountedProjectionChanges) {
        // Preserve physical changes recorded after this candidate took its
        // snapshot. This is required when accepted Motion advances an
        // unchanged command while a semantic successor is in flight.
        let mut late = pending.clone();
        late.apply(&self.observed);
        let mut remainder = self.remainder.clone();
        remainder.merge(&late);
        *pending = remainder;
    }
}

#[cfg(test)]
mod tests {
    use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

    use super::UiMountedProjectionChanges;

    #[test]
    fn reconciliation_consumes_only_its_current_surface_change() {
        let current_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let candidate_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let current_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let candidate_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let mut pending = UiMountedProjectionChanges::default();
        pending.mark_changed_instance(candidate_instance);
        pending.mark_changed_surface(current_surface);
        pending.mark_removed_surface(candidate_surface);

        let snapshot = pending.snapshot(7, 11);
        let reconciliation = snapshot
            .for_reconciliation(&[current_instance], &[current_surface])
            .unwrap();
        reconciliation.commit_into(&mut pending);
        let remainder = pending.snapshot(7, 11);

        assert_eq!(
            remainder.changed_instances().collect::<Vec<_>>(),
            vec![candidate_instance]
        );
        assert_eq!(
            remainder.removed_surfaces().collect::<Vec<_>>(),
            vec![candidate_surface]
        );
        assert_eq!(remainder.changed_surfaces().count(), 0);
    }

    #[test]
    fn reconciliation_rejects_changes_to_a_current_instance() {
        let current_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let current_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let mut pending = UiMountedProjectionChanges::default();
        pending.mark_changed_instance(current_instance);

        assert!(pending
            .snapshot(7, 11)
            .for_reconciliation(&[current_instance], &[current_surface])
            .is_none());
    }
}
