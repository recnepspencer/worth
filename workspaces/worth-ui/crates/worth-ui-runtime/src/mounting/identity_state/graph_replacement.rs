use std::collections::{BTreeMap, BTreeSet};

use crate::graph::{UiGraphAuthority, UiGraphNodeIdentity};
use worth_ui_host_contract::UiMountedInstanceIdentity;

use super::{MountedInstanceRecord, UiMountedIdentityState};
use crate::mounting::{UiMountedGraphWorldIdentity, UiMountedIdentityDenial};

impl UiMountedIdentityState {
    pub(crate) fn prepare_graph_replacement(
        &self,
    ) -> Result<UiMountedGraphWorldIdentity, UiMountedIdentityDenial> {
        Ok(UiMountedGraphWorldIdentity::new(super::next(
            &super::NEXT_WORLD,
        )?))
    }

    pub(crate) fn prepare_graph_replacement_successor(
        &self,
        graph: UiGraphAuthority<'_>,
    ) -> Result<Self, UiMountedIdentityDenial> {
        let next_world = self.prepare_graph_replacement()?;
        let semantic_revision = super::next(&super::NEXT_STATE_REVISION)?;
        let instances = surviving_instances(self, graph);
        let pending_projection_changes = replacement_projection_changes(self, &instances);
        let mut successor = Self {
            world_identity: next_world,
            host_session_identity: self.host_session_identity,
            semantic_surfaces: self.semantic_surfaces.clone(),
            bindings: self.bindings.clone(),
            by_graph: reverse_index(&instances),
            visible_order: retained_visible_order(self, &instances),
            mounted_instance_membership: persistent_membership(&instances),
            instances,
            retired_instances: self.retired_instances.clone(),
            retirement_order: self.retirement_order.clone(),
            current_frame: None,
            current_receipt_basis: None,
            current_projection: None,
            unprojected_semantic_predecessor: None,
            unprojected_appearance_predecessor: self.appearance_predecessor().cloned(),
            unprojected_pointer_predecessor: self.pointer_predecessor().cloned(),
            current_manifest: None,
            current_core: None,
            current_publication: None,
            current_trace_source: None,
            current_reuse_contract: None,
            pending_projection_changes,
            peak_qualified_layouts: self.peak_qualified_layouts,
            semantic_revision,
            binding_revision: self.binding_revision,
        };
        successor.remember_removed_instances(self);
        Ok(successor)
    }

    fn remember_removed_instances(&mut self, predecessor: &Self) {
        for identity in predecessor
            .instances
            .keys()
            .filter(|identity| !self.instances.contains_key(identity))
            .copied()
            .collect::<Vec<_>>()
        {
            self.remember_retirement(identity);
        }
    }
}

fn replacement_projection_changes(
    predecessor: &UiMountedIdentityState,
    instances: &BTreeMap<UiMountedInstanceIdentity, MountedInstanceRecord>,
) -> crate::mounting::UiMountedProjectionChanges {
    let mut changes = crate::mounting::UiMountedProjectionChanges::default();
    for identity in instances.keys().copied() {
        changes.mark_changed_instance(identity);
    }
    for identity in predecessor
        .instances
        .keys()
        .filter(|identity| !instances.contains_key(identity))
        .copied()
    {
        changes.mark_retired_instance(identity);
    }
    changes
}

fn surviving_instances(
    predecessor: &UiMountedIdentityState,
    graph: UiGraphAuthority<'_>,
) -> BTreeMap<UiMountedInstanceIdentity, MountedInstanceRecord> {
    predecessor
        .instances
        .iter()
        .filter(|(_, record)| {
            graph
                .lookup()
                .graph_node(record.basis.graph_node_identity())
                .is_some_and(|candidate| {
                    candidate.value().repeated_instance_basis()
                        == record.basis.repeated_instance_basis()
                })
        })
        .map(|(identity, record)| (*identity, record.clone()))
        .collect()
}

fn reverse_index(
    instances: &BTreeMap<UiMountedInstanceIdentity, MountedInstanceRecord>,
) -> BTreeMap<UiGraphNodeIdentity, BTreeSet<UiMountedInstanceIdentity>> {
    let mut by_graph = BTreeMap::<_, BTreeSet<_>>::new();
    for (identity, record) in instances {
        by_graph
            .entry(record.basis.graph_node_identity())
            .or_default()
            .insert(*identity);
    }
    by_graph
}

fn retained_visible_order(
    predecessor: &UiMountedIdentityState,
    instances: &BTreeMap<UiMountedInstanceIdentity, MountedInstanceRecord>,
) -> crate::runtime::persistent_index::UiPersistentOrder<UiMountedInstanceIdentity> {
    let retained = predecessor
        .visible_order
        .iter()
        .copied()
        .filter(|identity| instances.contains_key(identity))
        .collect::<Vec<_>>();
    let mut order = crate::runtime::persistent_index::UiPersistentOrder::default();
    order
        .replace_all(&retained)
        .expect("retained graph replacement order remains unique");
    order
}

fn persistent_membership(
    instances: &BTreeMap<UiMountedInstanceIdentity, MountedInstanceRecord>,
) -> crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedInstanceIdentity> {
    let mut membership = crate::runtime::persistent_index::UiPersistentOrdSet::<
        UiMountedInstanceIdentity,
    >::default();
    for identity in instances.keys().copied() {
        let inserted = membership.insert(identity);
        debug_assert!(inserted);
    }
    membership
}
