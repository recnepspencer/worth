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
    measurements: Rc<Cell<UiMountedAppearanceSelectionMeasurements>>,
}

#[derive(Clone, Copy, Default)]
struct UiMountedAppearanceSelectionMeasurements {
    materialized_contexts: usize,
    lifecycle_memberships_retired: usize,
    membership_key_probes: usize,
    membership_copied_avl_nodes: usize,
    membership_traversed_entries: usize,
    order_work: crate::mounting::spatial_index::UiMountedSpatialWork,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiMountedAppearanceSelectionCostReport {
    selected_instance_count: usize,
    materialized_context_count: usize,
    index_entries_touched: usize,
    lifecycle_memberships_retired: usize,
    membership_key_probes: usize,
    membership_copied_avl_nodes: usize,
    membership_traversed_entries: usize,
    order_work: crate::mounting::spatial_index::UiMountedSpatialWork,
    order_retained_bytes: usize,
    pointer_work: super::UiMountedPointerAffordanceWork,
}

impl UiMountedAppearanceProjectionSelection {
    pub(super) fn merge_physical_input_selection(
        &mut self,
        addition: Self,
        effective: &UiAppearanceInvalidationBatch,
    ) -> Option<()> {
        let work = self
            .index_entries_touched
            .checked_add(addition.index_entries_touched)?;
        let mut instances = self.selected_instances.to_vec();
        instances.extend_from_slice(&addition.selected_instances);
        instances.sort_unstable();
        instances.dedup();
        self.selected_instances = instances.into();
        self.basis = Some(effective.basis());
        self.batch_revision = Some(effective.revision());
        self.index_entries_touched = work;
        Some(())
    }

    #[cfg(test)]
    pub(super) fn with_independent_measurements(&self) -> Self {
        Self {
            measurements: Rc::new(Cell::new(self.measurements.get())),
            ..self.clone()
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            basis: None,
            batch_revision: None,
            selected_instances: Rc::from([]),
            retired_instances: Rc::from([]),
            index_entries_touched: 0,
            measurements: Rc::new(Cell::new(Default::default())),
        }
    }

    pub(crate) fn derive(
        state: &super::super::UiMountedIdentityState,
        requested_surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        index: Option<&crate::graph::UiGraphConsumedFactIndex>,
        invalidation: Option<&UiAppearanceInvalidationBatch>,
    ) -> Option<Self> {
        let (basis, batch_revision, selected_instances, index_entries_touched) =
            match invalidation {
                Some(batch) => {
                    let affected =
                        state.try_projection_instances_for_graph_nodes(batch.graph_consumers())?;
                    let mut selected = affected
                        .instances()
                        .iter()
                        .copied()
                        .filter(|instance| {
                            state.projection_instance(*instance).is_some_and(|view| {
                                requested_surfaces
                                    .contains(&view.basis().semantic_surface_identity())
                            })
                        })
                        .collect::<Vec<_>>();
                    selected.extend(batch.mounted_consumers().iter().filter_map(
                        |(node, instance)| {
                            let view = state.projection_instance(*instance)?;
                            (view.basis().graph_node_identity() == *node
                                && requested_surfaces
                                    .contains(&view.basis().semantic_surface_identity()))
                            .then_some(*instance)
                        },
                    ));
                    selected.sort_unstable();
                    selected.dedup();
                    (
                        Some(batch.basis()),
                        Some(batch.revision()),
                        selected.into(),
                        affected.index_entries_touched() + batch.mounted_consumers().len(),
                    )
                }
                None => (index.map(|index| index.basis()), None, Rc::from([]), 0),
            };
        Some(Self {
            basis,
            batch_revision,
            selected_instances,
            retired_instances: Rc::from([]),
            index_entries_touched,
            measurements: Rc::new(Cell::new(Default::default())),
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

    pub(in crate::mounting) fn matches_consumer_basis(
        &self,
        basis: crate::graph::UiGraphFactIndexBasis,
    ) -> bool {
        self.basis == Some(basis)
    }

    pub(crate) fn matches_batch(&self, batch: &UiAppearanceInvalidationBatch) -> bool {
        self.basis == Some(batch.basis()) && self.batch_revision == Some(batch.revision())
    }

    pub(crate) const fn index_entries_touched(&self) -> usize {
        self.index_entries_touched
    }

    pub(crate) fn record_materialized_contexts(&self, count: usize) {
        let mut measurements = self.measurements.get();
        measurements.materialized_contexts = count;
        self.measurements.set(measurements);
    }

    pub(crate) fn record_lifecycle_memberships_retired(&self, count: usize) {
        let mut measurements = self.measurements.get();
        measurements.lifecycle_memberships_retired = measurements
            .lifecycle_memberships_retired
            .saturating_add(count);
        self.measurements.set(measurements);
    }

    pub(crate) fn record_membership_work(
        &self,
        work: super::frame_storage::UiMountedAppearanceMembershipWork,
    ) {
        let mut measurements = self.measurements.get();
        measurements.membership_key_probes = measurements
            .membership_key_probes
            .saturating_add(work.key_probes());
        measurements.membership_copied_avl_nodes = measurements
            .membership_copied_avl_nodes
            .saturating_add(work.copied_avl_nodes());
        measurements.membership_traversed_entries = measurements
            .membership_traversed_entries
            .saturating_add(work.traversed_entries());
        self.measurements.set(measurements);
    }

    pub(crate) fn record_order_work(
        &self,
        work: crate::mounting::spatial_index::UiMountedSpatialWork,
    ) {
        let mut measurements = self.measurements.get();
        measurements.order_work.merge(work);
        self.measurements.set(measurements);
    }

    pub(crate) fn cost_report(
        &self,
        order_retained_bytes: usize,
    ) -> UiMountedAppearanceSelectionCostReport {
        let measurements = self.measurements.get();
        UiMountedAppearanceSelectionCostReport {
            selected_instance_count: self.selected_instances.len(),
            materialized_context_count: measurements.materialized_contexts,
            index_entries_touched: self.index_entries_touched,
            lifecycle_memberships_retired: measurements.lifecycle_memberships_retired,
            membership_key_probes: measurements.membership_key_probes,
            membership_copied_avl_nodes: measurements.membership_copied_avl_nodes,
            membership_traversed_entries: measurements.membership_traversed_entries,
            order_work: measurements.order_work,
            order_retained_bytes,
            pointer_work: Default::default(),
        }
    }
}

impl UiMountedAppearanceSelectionCostReport {
    pub(in crate::mounting) fn with_pointer_work(
        mut self,
        work: super::UiMountedPointerAffordanceWork,
    ) -> Self {
        self.pointer_work = work;
        self
    }

    pub const fn pointer_work(self) -> super::UiMountedPointerAffordanceWork {
        self.pointer_work
    }
    pub const fn order_retained_bytes(self) -> usize {
        self.order_retained_bytes
    }
    pub const fn order_work(self) -> crate::mounting::spatial_index::UiMountedSpatialWork {
        self.order_work
    }
    pub const fn selected_instance_count(self) -> usize {
        self.selected_instance_count
    }

    pub const fn materialized_context_count(self) -> usize {
        self.materialized_context_count
    }

    pub const fn index_entries_touched(self) -> usize {
        self.index_entries_touched
    }

    pub const fn lifecycle_memberships_retired(self) -> usize {
        self.lifecycle_memberships_retired
    }

    pub const fn membership_key_probes(self) -> usize {
        self.membership_key_probes
    }

    pub const fn membership_copied_avl_nodes(self) -> usize {
        self.membership_copied_avl_nodes
    }

    pub const fn membership_traversed_entries(self) -> usize {
        self.membership_traversed_entries
    }
}
