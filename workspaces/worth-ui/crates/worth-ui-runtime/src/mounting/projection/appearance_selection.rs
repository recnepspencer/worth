use std::cell::Cell;
use std::rc::Rc;

use crate::runtime::appearance::UiAppearanceInvalidationBatch;

#[derive(Clone)]
pub(crate) struct UiMountedAppearanceProjectionSelection {
    basis: Option<crate::graph::UiGraphFactIndexBasis>,
    batch_revision: Option<u64>,
    selected_instances: Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    retired_instances: Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    index_entries_touched: usize,
    materialized_contexts: Rc<Cell<usize>>,
    lifecycle_memberships_retired: Rc<Cell<usize>>,
    membership_key_probes: Rc<Cell<usize>>,
    membership_copied_avl_nodes: Rc<Cell<usize>>,
    membership_traversed_entries: Rc<Cell<usize>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceSelectionCostReport {
    selected_instance_count: usize,
    materialized_context_count: usize,
    index_entries_touched: usize,
    lifecycle_memberships_retired: usize,
    membership_key_probes: usize,
    membership_copied_avl_nodes: usize,
    membership_traversed_entries: usize,
}

impl UiMountedAppearanceProjectionSelection {
    pub(crate) fn empty() -> Self {
        Self {
            basis: None,
            batch_revision: None,
            selected_instances: Rc::from([]),
            retired_instances: Rc::from([]),
            index_entries_touched: 0,
            materialized_contexts: Rc::new(Cell::new(0)),
            lifecycle_memberships_retired: Rc::new(Cell::new(0)),
            membership_key_probes: Rc::new(Cell::new(0)),
            membership_copied_avl_nodes: Rc::new(Cell::new(0)),
            membership_traversed_entries: Rc::new(Cell::new(0)),
        }
    }

    pub(crate) fn derive(
        state: &super::super::UiMountedIdentityState,
        requested_surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        invalidation: Option<&UiAppearanceInvalidationBatch>,
    ) -> Option<Self> {
        let (basis, batch_revision, selected_instances, index_entries_touched) = match invalidation
        {
            Some(batch) => {
                let affected = state.try_projection_instances_for_graph_nodes(batch.consumers())?;
                let selected = affected
                    .instances()
                    .iter()
                    .copied()
                    .filter(|instance| {
                        state.projection_instance(*instance).is_some_and(|view| {
                            requested_surfaces.contains(&view.basis().semantic_surface_identity())
                        })
                    })
                    .collect::<Vec<_>>()
                    .into();
                (
                    Some(batch.basis()),
                    Some(batch.revision()),
                    selected,
                    affected.index_entries_touched(),
                )
            }
            None => (None, None, Rc::from([]), 0),
        };
        Some(Self {
            basis,
            batch_revision,
            selected_instances,
            retired_instances: Rc::from([]),
            index_entries_touched,
            materialized_contexts: Rc::new(Cell::new(0)),
            lifecycle_memberships_retired: Rc::new(Cell::new(0)),
            membership_key_probes: Rc::new(Cell::new(0)),
            membership_copied_avl_nodes: Rc::new(Cell::new(0)),
            membership_traversed_entries: Rc::new(Cell::new(0)),
        })
    }

    pub(crate) fn set_retired_instances(
        &mut self,
        mut instances: Vec<worth_ui_host_contract::UiMountedInstanceIdentity>,
    ) {
        instances.sort_unstable();
        instances.dedup();
        self.retired_instances = instances.into();
    }

    pub(crate) fn selected_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        &self.selected_instances
    }

    pub(crate) fn retired_instances(&self) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        &self.retired_instances
    }

    pub(crate) fn matches_batch(&self, batch: &UiAppearanceInvalidationBatch) -> bool {
        self.basis == Some(batch.basis()) && self.batch_revision == Some(batch.revision())
    }

    pub(crate) const fn index_entries_touched(&self) -> usize {
        self.index_entries_touched
    }

    pub(crate) fn record_materialized_contexts(&self, count: usize) {
        self.materialized_contexts.set(count);
    }

    pub(crate) fn record_lifecycle_memberships_retired(&self, count: usize) {
        self.lifecycle_memberships_retired.set(
            self.lifecycle_memberships_retired
                .get()
                .saturating_add(count),
        );
    }

    pub(crate) fn record_membership_work(
        &self,
        work: super::frame_storage::UiMountedAppearanceMembershipWork,
    ) {
        self.membership_key_probes.set(
            self.membership_key_probes
                .get()
                .saturating_add(work.key_probes()),
        );
        self.membership_copied_avl_nodes.set(
            self.membership_copied_avl_nodes
                .get()
                .saturating_add(work.copied_avl_nodes()),
        );
        self.membership_traversed_entries.set(
            self.membership_traversed_entries
                .get()
                .saturating_add(work.traversed_entries()),
        );
    }

    pub(crate) fn cost_report(&self) -> UiMountedAppearanceSelectionCostReport {
        UiMountedAppearanceSelectionCostReport {
            selected_instance_count: self.selected_instances.len(),
            materialized_context_count: self.materialized_contexts.get(),
            index_entries_touched: self.index_entries_touched,
            lifecycle_memberships_retired: self.lifecycle_memberships_retired.get(),
            membership_key_probes: self.membership_key_probes.get(),
            membership_copied_avl_nodes: self.membership_copied_avl_nodes.get(),
            membership_traversed_entries: self.membership_traversed_entries.get(),
        }
    }
}

impl UiMountedAppearanceSelectionCostReport {
    pub(crate) const fn selected_instance_count(self) -> usize {
        self.selected_instance_count
    }

    pub(crate) const fn materialized_context_count(self) -> usize {
        self.materialized_context_count
    }

    pub(crate) const fn index_entries_touched(self) -> usize {
        self.index_entries_touched
    }

    pub(crate) const fn lifecycle_memberships_retired(self) -> usize {
        self.lifecycle_memberships_retired
    }

    pub(crate) const fn membership_key_probes(self) -> usize {
        self.membership_key_probes
    }

    pub(crate) const fn membership_copied_avl_nodes(self) -> usize {
        self.membership_copied_avl_nodes
    }

    pub(crate) const fn membership_traversed_entries(self) -> usize {
        self.membership_traversed_entries
    }
}
