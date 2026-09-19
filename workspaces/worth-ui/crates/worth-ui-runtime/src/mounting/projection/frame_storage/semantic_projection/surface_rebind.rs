use worth_ui_host_contract::{UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration};

use super::UiMountedSemanticProjection;

impl UiMountedSemanticProjection {
    pub(in crate::mounting::projection) fn rebind_surface_allocations(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
    ) {
        let instances = self
            .order
            .iter()
            .copied()
            .filter(|instance| {
                self.nodes
                    .get(instance)
                    .is_some_and(|node| node.receipt.semantic_surface() == surface)
            })
            .collect::<Vec<_>>();
        for instance in instances {
            let Some(mut node) = self.nodes.get(&instance).cloned() else {
                continue;
            };
            node.receipt.rebind_allocation(binding);
            node.occurrence_allocation =
                crate::mounting::projection::node_receipt::rebound_allocation(
                    node.occurrence_allocation,
                    binding,
                    node.receipt.incarnation(),
                );
            node.appearance_geometry.allocation =
                crate::mounting::projection::node_receipt::rebound_allocation(
                    node.appearance_geometry.allocation,
                    binding,
                    node.receipt.incarnation(),
                );
            self.nodes.insert(instance, node);
        }
    }
}
