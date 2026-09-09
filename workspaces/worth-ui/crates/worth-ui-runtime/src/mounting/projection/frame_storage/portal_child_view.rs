use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPortalOverlayMechanic, UiSemanticSurfaceIdentity,
};

use super::{UiMountedProjectionDenial, UiMountedProjectionFrame};

#[derive(Clone, Copy)]
pub(super) enum UiMountedPortalChildPresentation {
    Ordinary,
    Suppressed,
    Presented(UiMountedPortalOverlayMechanic),
}

impl UiMountedProjectionFrame {
    pub(in crate::mounting) fn portal_owner_for_child(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Option<(crate::graph::UiGraphNodeIdentity, UiMountedInstanceIdentity)> {
        let child = self.semantic.node(instance)?;
        let owner_component = child.portal_child_owner.as_ref()?;
        let mut matched = None;
        for input in self.portal_overlays.iter().copied() {
            let owner = self.semantic.node(input.owner())?;
            if owner.receipt.semantic_surface() != child.receipt.semantic_surface()
                || owner.component_id.as_ref() != Some(owner_component)
            {
                continue;
            }
            if matched.is_some() {
                return None;
            }
            matched = Some((owner.receipt.graph_node(), input.owner()));
        }
        matched
    }

    pub(in crate::mounting) fn participates_in_focus(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> bool {
        let Some(node) = self.semantic.node(instance) else {
            return false;
        };
        let Some(owner_component) = node.portal_child_owner.as_ref() else {
            return true;
        };
        self.portal_overlays.iter().copied().any(|input| {
            if !matches!(
                input.lifecycle(),
                crate::runtime::portal::UiPortalLifecyclePosture::Open
                    | crate::runtime::portal::UiPortalLifecyclePosture::Visible
            ) {
                return false;
            }
            self.semantic.node(input.owner()).is_some_and(|owner| {
                owner.receipt.semantic_surface() == node.receipt.semantic_surface()
                    && owner.component_id.as_ref() == Some(owner_component)
            })
        })
    }

    pub(super) fn portal_child_presentation(
        &self,
        instance: UiMountedInstanceIdentity,
        surface: UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<UiMountedPortalChildPresentation, UiMountedProjectionDenial> {
        self.portal_child_presentation_with_work(instance, surface, binding)
            .map(|(presentation, _, _)| presentation)
    }

    /// Returns the presentation, actual node-index probes, and overlay rows visited.
    pub(super) fn portal_child_presentation_with_work(
        &self,
        instance: UiMountedInstanceIdentity,
        surface: UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<(UiMountedPortalChildPresentation, usize, usize), UiMountedProjectionDenial> {
        let (node, mut probes) = self.semantic.nodes.get_with_probes(&instance);
        let Some(node) = node else {
            return Ok((UiMountedPortalChildPresentation::Ordinary, probes, 0));
        };
        let Some(owner_component) = node.portal_child_owner.as_ref() else {
            return Ok((UiMountedPortalChildPresentation::Ordinary, probes, 0));
        };
        let mut matched = None;
        for input in self.portal_overlays.iter().copied() {
            let (owner, work) = self.semantic.nodes.get_with_probes(&input.owner());
            probes = probes
                .checked_add(work)
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
            let owner = owner.ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            if owner.receipt.semantic_surface() != surface
                || owner.component_id.as_ref() != Some(owner_component)
            {
                continue;
            }
            if matched.is_some() {
                return Err(UiMountedProjectionDenial::AmbiguousPortalChildOwner);
            }
            let receipt = self
                .receipt_basis
                .receipt_for(input.owner())
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            matched = Some(
                input
                    .mechanic_for(self.frame, surface, binding, receipt)
                    .map_err(UiMountedProjectionDenial::PortalOverlayCompletion)?,
            );
        }
        Ok((
            matched.map_or(
                UiMountedPortalChildPresentation::Suppressed,
                UiMountedPortalChildPresentation::Presented,
            ),
            probes,
            self.portal_overlays.len(),
        ))
    }
}
