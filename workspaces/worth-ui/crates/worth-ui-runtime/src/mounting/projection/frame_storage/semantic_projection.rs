use worth_ui_host_contract::{
    UiMountedProjectionAudience, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

use super::super::UiMountedNodeReceipt;

mod portal_children;
mod surface_rebind;

#[derive(Clone)]
pub(in crate::mounting) struct UiMountedProjectionNodeRecord {
    pub(in crate::mounting::projection) receipt: UiMountedNodeReceipt,
    pub(in crate::mounting::projection) plan_index: Option<u32>,
    pub(in crate::mounting::projection) surface_paint_order: Option<u32>,
    pub(in crate::mounting::projection) has_appearance_attachment: bool,
    pub(in crate::mounting::projection) appearance_clip:
        super::super::appearance::UiMountedAppearanceClip,
    pub(in crate::mounting::projection) appearance_geometry: super::UiMountedAppearanceGeometry,
    pub(in crate::mounting::projection) occurrence_allocation:
        worth_ui_host_contract::UiMountedAllocationProjection,
    pub(in crate::mounting::projection) static_paint:
        Option<super::super::static_paint::UiMountedStaticPaintSeed>,
    pub(in crate::mounting::projection) semantic_text:
        Option<super::super::semantic_text::UiMountedSemanticTextSeed>,
    pub(in crate::mounting::projection) hit_test:
        Option<super::super::hit_test::UiMountedHitTestSeed>,
    pub(in crate::mounting) focus_support: crate::capability::ComponentFocusSupport,
    pub(in crate::mounting) focus_scope: Option<super::super::UiMountedFocusScope>,
    pub(in crate::mounting) focus_container_owner: Option<crate::graph::UiGraphNodeIdentity>,
    pub(in crate::mounting::projection) component_id: Option<crate::capability::ComponentId>,
    pub(in crate::mounting::projection) portal_child_owner: Option<crate::capability::ComponentId>,
}

impl UiMountedProjectionNodeRecord {
    pub(in crate::mounting) const fn receipt(&self) -> &UiMountedNodeReceipt {
        &self.receipt
    }

    pub(in crate::mounting::projection) fn presentation_allocation(
        &self,
    ) -> worth_ui_host_contract::UiMountedAllocationProjection {
        if self.portal_child_owner.is_some() {
            self.occurrence_allocation
        } else {
            self.receipt.allocation()
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::mounting::projection) struct UiMountedProjectionSurface {
    pub(in crate::mounting::projection) surface: UiSemanticSurfaceIdentity,
    pub(in crate::mounting::projection) binding: UiSurfaceBindingGeneration,
    pub(in crate::mounting::projection) audience: UiMountedProjectionAudience,
}

#[derive(Clone)]
pub(in crate::mounting) struct UiMountedSemanticProjection {
    pub(super) nodes: crate::runtime::persistent_index::UiPersistentOrdMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        UiMountedProjectionNodeRecord,
    >,
    pub(super) order: crate::runtime::persistent_index::UiPersistentOrder<
        worth_ui_host_contract::UiMountedInstanceIdentity,
    >,
    membership: crate::runtime::persistent_index::UiPersistentOrdSet<
        worth_ui_host_contract::UiMountedInstanceIdentity,
    >,
    portal_children: portal_children::UiMountedPortalChildMembership,
    semantic_surfaces:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiSemanticSurfaceIdentity>,
    binding_by_surface: crate::runtime::persistent_index::UiPersistentOrdMap<
        UiSemanticSurfaceIdentity,
        UiSurfaceBindingGeneration,
    >,
    pub(super) surfaces: crate::runtime::persistent_index::UiPersistentOrdMap<
        UiSurfaceBindingGeneration,
        UiMountedProjectionSurface,
    >,
    projection_input_capacity: usize,
    projection_inputs: crate::runtime::persistent_index::UiPersistentSlotTrie<
        worth_ui_query_binding::UiProjectionInputFactReference,
    >,
}

impl UiMountedSemanticProjection {
    #[cfg(test)]
    pub(in crate::mounting::projection) fn initial(
        nodes: Vec<UiMountedProjectionNodeRecord>,
        surfaces: Vec<UiMountedProjectionSurface>,
    ) -> Self {
        Self::build_initial(nodes, surfaces).0
    }

    pub(in crate::mounting::projection) fn build_initial(
        nodes: Vec<UiMountedProjectionNodeRecord>,
        surfaces: Vec<UiMountedProjectionSurface>,
    ) -> (
        Self,
        crate::runtime::persistent_index::UiPersistentIndexMutationWork,
    ) {
        let mut order = crate::runtime::persistent_index::UiPersistentOrder::default();
        for node in &nodes {
            order
                .append(node.receipt.mounted_instance())
                .expect("initial semantic projection identities are unique");
        }
        let mut node_index = crate::runtime::persistent_index::UiPersistentOrdMap::default();
        let mut membership = crate::runtime::persistent_index::UiPersistentOrdSet::default();
        let mut portal_children = portal_children::UiMountedPortalChildMembership::default();
        let mut portal_work =
            crate::runtime::persistent_index::UiPersistentIndexMutationWork::default();
        for node in nodes {
            let instance = node.receipt.mounted_instance();
            portal_work
                .merge(portal_children.replace(None, Some(&node)))
                .expect("initial membership work fits address space");
            node_index.insert(instance, node);
            membership.insert(instance);
        }
        let mut surface_index = crate::runtime::persistent_index::UiPersistentOrdMap::default();
        let mut binding_by_surface =
            crate::runtime::persistent_index::UiPersistentOrdMap::default();
        let mut semantic_surfaces = crate::runtime::persistent_index::UiPersistentOrdSet::default();
        for surface in surfaces {
            semantic_surfaces.insert(surface.surface);
            binding_by_surface.insert(surface.surface, surface.binding);
            surface_index.insert(surface.binding, surface);
        }
        (
            Self {
                nodes: node_index,
                order,
                membership,
                portal_children,
                semantic_surfaces,
                binding_by_surface,
                surfaces: surface_index,
                projection_input_capacity: 0,
                projection_inputs: Default::default(),
            },
            portal_work,
        )
    }

    pub(in crate::mounting::projection) fn membership(
        &self,
    ) -> crate::runtime::persistent_index::UiPersistentOrdSet<
        worth_ui_host_contract::UiMountedInstanceIdentity,
    > {
        self.membership.clone()
    }

    pub(in crate::mounting::projection) fn supports_surfaces(
        &self,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> bool {
        surfaces.len() == self.semantic_surfaces.len()
            && surfaces
                .iter()
                .all(|surface| self.semantic_surfaces.contains_with_probes(surface).0)
    }

    pub(in crate::mounting::projection) fn contains(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.membership.contains_with_probes(&instance).0
    }

    pub(in crate::mounting::projection) fn node(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<&UiMountedProjectionNodeRecord> {
        self.nodes.get(&instance)
    }

    pub(in crate::mounting::projection) fn node_with_probes(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> (Option<&UiMountedProjectionNodeRecord>, usize) {
        self.nodes.get_with_probes(&instance)
    }

    pub(in crate::mounting::projection) fn insert_node(
        &mut self,
        node: UiMountedProjectionNodeRecord,
    ) -> crate::runtime::persistent_index::UiPersistentIndexMutationWork {
        let instance = node.receipt.mounted_instance();
        let (previous, probes) = self.nodes.get_with_probes(&instance);
        let mut work =
            crate::runtime::persistent_index::UiPersistentIndexMutationWork::with_key_probes(
                probes,
            );
        work.merge(self.portal_children.replace(previous, Some(&node)))
            .expect("mounted index work fits address space");
        work.merge(self.membership.insert_with_work(instance).1)
            .expect("mounted index work fits address space");
        if previous.is_none() {
            work.merge(
                self.order
                    .append(instance)
                    .expect("new mounted projection identity appends once"),
            )
            .expect("mounted index work fits address space");
        }
        work.merge(self.nodes.insert_with_work(instance, node))
            .expect("mounted index work fits address space");
        work
    }

    pub(in crate::mounting::projection) fn remove_node(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> crate::runtime::persistent_index::UiPersistentIndexMutationWork {
        let (previous, probes) = self.nodes.get_with_probes(&instance);
        let mut work =
            crate::runtime::persistent_index::UiPersistentIndexMutationWork::with_key_probes(
                probes,
            );
        work.merge(self.portal_children.replace(previous, None))
            .expect("mounted index work fits address space");
        work.merge(self.membership.remove_with_work(&instance).1)
            .expect("mounted index work fits address space");
        if previous.is_some() {
            work.merge(
                self.order
                    .remove(instance)
                    .expect("projected mounted identity owns one order row"),
            )
            .expect("mounted index work fits address space");
        }
        work.merge(self.nodes.remove_with_work(&instance).1)
            .expect("mounted index work fits address space");
        work
    }

    pub(in crate::mounting::projection) fn replace_order(
        &mut self,
        order: Vec<worth_ui_host_contract::UiMountedInstanceIdentity>,
    ) {
        self.order
            .replace_all(&order)
            .expect("validated semantic order contains unique identities");
    }

    pub(in crate::mounting::projection) fn replace_order_snapshot(
        &mut self,
        order: crate::runtime::persistent_index::UiPersistentOrder<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
    ) {
        self.order = order;
    }

    pub(in crate::mounting::projection) fn replace_surface(
        &mut self,
        surface: UiMountedProjectionSurface,
    ) -> crate::runtime::persistent_index::UiPersistentIndexMutationWork {
        if let Some(previous) = self.binding_by_surface.get(&surface.surface).copied() {
            self.surfaces.remove(&previous);
        }
        self.semantic_surfaces.insert(surface.surface);
        self.binding_by_surface
            .insert(surface.surface, surface.binding);
        self.surfaces.insert_with_work(surface.binding, surface)
    }

    pub(in crate::mounting::projection) fn remove_surface(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) -> crate::runtime::persistent_index::UiPersistentIndexMutationWork {
        self.semantic_surfaces.remove_with_work(&surface);
        let binding = self.binding_by_surface.get(&surface).copied();
        self.binding_by_surface.remove(&surface);
        binding.map_or_else(Default::default, |binding| {
            self.surfaces.remove_with_work(&binding).1
        })
    }

    pub(in crate::mounting::projection) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(in crate::mounting::projection) fn mounted_instances(
        &self,
    ) -> impl ExactSizeIterator<Item = worth_ui_host_contract::UiMountedInstanceIdentity> + '_ {
        self.order.iter().copied()
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn nodes_in_order(
        &self,
    ) -> impl Iterator<Item = &UiMountedProjectionNodeRecord> {
        self.order.iter().map(|instance| {
            self.nodes
                .get(instance)
                .expect("mounted semantic order names an indexed node")
        })
    }

    pub(in crate::mounting) fn node_receipt_with_probes(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> (Option<&UiMountedNodeReceipt>, usize) {
        let (record, probes) = self.nodes.get_with_probes(&mounted_instance);
        (record.map(|record| &record.receipt), probes)
    }

    pub(in crate::mounting) fn nodes_in_mounted_order(
        &self,
    ) -> impl ExactSizeIterator<Item = &UiMountedProjectionNodeRecord> {
        self.order.iter().map(|instance| {
            self.nodes
                .get(instance)
                .expect("mounted semantic order names an indexed node")
        })
    }

    pub(in crate::mounting) fn retained_structural_bytes(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.nodes.retained_structural_bytes()?)?
            .checked_add(self.order.retained_structural_bytes()?)?
            .checked_add(self.membership.retained_structural_bytes()?)?
            .checked_add(self.portal_children.retained_structural_bytes()?)?
            .checked_add(self.semantic_surfaces.retained_structural_bytes()?)?
            .checked_add(self.binding_by_surface.retained_structural_bytes()?)?
            .checked_add(self.surfaces.retained_structural_bytes()?)?
            .checked_add(self.projection_inputs.retained_structural_bytes()?)
    }

    pub(in crate::mounting::projection) fn surface_instance_count(
        &self,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> usize {
        self.order
            .iter()
            .filter(|instance| {
                self.nodes
                    .get(instance)
                    .is_some_and(|node| surfaces.contains(&node.receipt.semantic_surface()))
            })
            .count()
    }

    pub(in crate::mounting::projection) fn surface_for(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<UiMountedProjectionSurface> {
        self.surface_for_with_probes(surface).0
    }

    pub(in crate::mounting::projection) fn surface_for_with_probes(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> (Option<UiMountedProjectionSurface>, usize) {
        let (binding, probes) = self.binding_by_surface.get_with_probes(&surface);
        let Some(binding) = binding else {
            return (None, probes);
        };
        let (surface, surface_probes) = self.surfaces.get_with_probes(binding);
        (surface.copied(), probes + surface_probes)
    }
}

#[path = "semantic_projection_inputs.rs"]
mod projection_inputs;
