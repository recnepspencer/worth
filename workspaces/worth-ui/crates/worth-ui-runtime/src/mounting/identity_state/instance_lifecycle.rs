use std::collections::BTreeSet;

use crate::graph::{UiGraphAuthority, UiGraphNodeIdentity};
use worth_ui_host_contract::{
    UiMountIncarnation, UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

use super::{MountedInstanceRecord, UiMountedIdentityState};
use crate::mounting::{UiMountedGraphNodeHandle, UiMountedIdentityBasis, UiMountedIdentityDenial};

const MOUNTED_CLOSURE_LIMIT: usize = 4_097;
const GRAPH_NODE_MOUNT_LIMIT: usize = 1_024;

impl UiMountedIdentityState {
    pub(crate) fn graph_occurrence_count(&self, graph_node: UiGraphNodeIdentity) -> usize {
        self.by_graph.get(&graph_node).map_or(0, BTreeSet::len)
    }

    pub(crate) fn mark_occurrence_geometry_changed(
        &mut self,
        instances: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial> {
        if instances
            .iter()
            .any(|instance| !self.instances.contains_key(instance))
        {
            return Err(UiMountedIdentityDenial::UnknownMountedInstance);
        }
        let semantic_revision = super::next(&super::NEXT_STATE_REVISION)?;
        for instance in instances {
            self.pending_projection_changes
                .mark_changed_instance(*instance);
        }
        self.semantic_revision = semantic_revision;
        Ok(())
    }

    pub(crate) fn mark_motion_appearance_changed(
        &mut self,
        instances: &[UiMountedInstanceIdentity],
    ) {
        for instance in instances {
            if self.instances.contains_key(instance) {
                self.pending_projection_changes
                    .mark_appearance_input_changed(*instance);
            }
        }
    }

    pub(crate) fn graph_node_handle(
        &self,
        graph: UiGraphAuthority<'_>,
        graph_node_identity: UiGraphNodeIdentity,
    ) -> Result<UiMountedGraphNodeHandle, UiMountedIdentityDenial> {
        graph
            .lookup()
            .graph_node(graph_node_identity)
            .ok_or(UiMountedIdentityDenial::UnknownGraphNode)?;
        Ok(UiMountedGraphNodeHandle::new(
            self.world_identity,
            graph_node_identity,
        ))
    }

    pub(crate) fn mount(
        &mut self,
        graph: UiGraphAuthority<'_>,
        handle: UiMountedGraphNodeHandle,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiMountedInstanceIdentity, UiMountedIdentityDenial> {
        self.require_handle(handle)?;
        self.require_surface(surface)?;
        self.require_mount_capacity(handle)?;
        let graph_node = graph
            .lookup()
            .graph_node(handle.graph_node_identity())
            .ok_or(UiMountedIdentityDenial::UnknownGraphNode)?;
        let incarnation = UiMountIncarnation::mint_unbound()
            .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        let identity = UiMountedInstanceIdentity::mint_unbound()
            .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        let semantic_revision = super::next(&super::NEXT_STATE_REVISION)?;
        let basis = UiMountedIdentityBasis::new(
            handle.graph_node_identity(),
            graph_node.value().repeated_instance_basis().clone(),
            surface,
            incarnation,
        );
        self.commit_mount(identity, basis);
        self.pending_projection_changes
            .mark_changed_instance(identity);
        self.semantic_revision = semantic_revision;
        Ok(identity)
    }

    pub(crate) fn unmount(
        &mut self,
        identity: UiMountedInstanceIdentity,
        occurrence_geometry_affected: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial> {
        let record = self.instances.get(&identity).cloned().ok_or_else(|| {
            if self.retired_instances.contains(&identity) {
                UiMountedIdentityDenial::RetiredMountedInstance
            } else {
                UiMountedIdentityDenial::UnknownMountedInstance
            }
        })?;
        if occurrence_geometry_affected
            .iter()
            .any(|instance| !self.instances.contains_key(instance))
        {
            return Err(UiMountedIdentityDenial::UnknownMountedInstance);
        }
        let semantic_revision = super::next(&super::NEXT_STATE_REVISION)?;
        self.instances.remove(&identity);
        if let Some(instances) = self.by_graph.get_mut(&record.basis.graph_node_identity()) {
            instances.remove(&identity);
        }
        self.visible_order
            .remove(identity)
            .expect("mounted identity belongs to authored order");
        let removed = self
            .mounted_instance_membership
            .remove_with_work(&identity)
            .0;
        debug_assert!(removed);
        if let Some(current) = self.current_receipt_basis.as_mut() {
            current.remove(identity);
        }
        self.pending_projection_changes
            .mark_retired_instance(identity);
        for affected in occurrence_geometry_affected {
            if *affected != identity {
                self.pending_projection_changes
                    .mark_changed_instance(*affected);
            }
        }
        self.remember_retirement(identity);
        self.semantic_revision = semantic_revision;
        Ok(())
    }

    pub(crate) fn reorder(
        &mut self,
        order: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial> {
        let requested = order.iter().copied().collect::<BTreeSet<_>>();
        let current = self.visible_order.iter().copied().collect::<BTreeSet<_>>();
        if requested != current || requested.len() != order.len() {
            return Err(UiMountedIdentityDenial::ReorderMembershipMismatch);
        }
        let semantic_revision = super::next(&super::NEXT_STATE_REVISION)?;
        self.visible_order
            .replace_all(order)
            .map_err(|_| UiMountedIdentityDenial::ReorderMembershipMismatch)?;
        self.pending_projection_changes.mark_order_changed(order);
        self.semantic_revision = semantic_revision;
        Ok(())
    }

    pub(super) fn require_handle(
        &self,
        handle: UiMountedGraphNodeHandle,
    ) -> Result<(), UiMountedIdentityDenial> {
        if handle.world_identity() != self.world_identity {
            return Err(UiMountedIdentityDenial::ForeignGraphWorld);
        }
        Ok(())
    }

    fn require_mount_capacity(
        &self,
        handle: UiMountedGraphNodeHandle,
    ) -> Result<(), UiMountedIdentityDenial> {
        if self.instances.len() >= MOUNTED_CLOSURE_LIMIT {
            return Err(UiMountedIdentityDenial::MountedClosureCapacityExceeded);
        }
        if self
            .by_graph
            .get(&handle.graph_node_identity())
            .is_some_and(|instances| instances.len() >= GRAPH_NODE_MOUNT_LIMIT)
        {
            return Err(UiMountedIdentityDenial::GraphNodeMountCapacityExceeded);
        }
        Ok(())
    }

    fn commit_mount(&mut self, identity: UiMountedInstanceIdentity, basis: UiMountedIdentityBasis) {
        let graph_node = basis.graph_node_identity();
        self.instances
            .insert(identity, MountedInstanceRecord { basis });
        self.by_graph
            .entry(graph_node)
            .or_default()
            .insert(identity);
        self.visible_order
            .append(identity)
            .expect("mounted order capacity follows identity capacity");
        let inserted = self.mounted_instance_membership.insert(identity);
        debug_assert!(inserted);
    }
}
