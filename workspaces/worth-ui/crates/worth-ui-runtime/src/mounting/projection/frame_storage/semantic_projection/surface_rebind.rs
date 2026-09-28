use worth_ui_host_contract::{UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration};

use super::UiMountedSemanticProjection;

impl UiMountedSemanticProjection {
    #[expect(
        clippy::disallowed_methods,
        reason = "rebinding a surface changes only its allocations' binding, never where they are shown"
    )]
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
            let incarnation = node.receipt.incarnation();
            let rebound = |allocation| {
                crate::mounting::projection::node_receipt::rebound_allocation(
                    allocation,
                    binding,
                    incarnation,
                )
            };
            node.occurrence_allocation = node.occurrence_allocation.map(rebound);
            node.appearance_geometry.allocation = node.appearance_geometry.allocation.map(rebound);
            self.nodes.insert(instance, node);
        }
    }
}
