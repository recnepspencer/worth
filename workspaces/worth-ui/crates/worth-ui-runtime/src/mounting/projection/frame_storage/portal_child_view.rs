use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

use super::{UiMountedProjectionDenial, UiMountedProjectionFrame};
use crate::mounting::projection::placement::{UiMountedPlacement, UiPortalPresentation};

impl UiMountedProjectionFrame {
    pub(in crate::mounting) fn portal_owner_for_child(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Option<(crate::graph::UiGraphNodeIdentity, UiMountedInstanceIdentity)> {
        let child = self.semantic.node(instance)?;
        let owner_component = child.portal_child_owner.as_ref()?;
        let mut matched = None;
        for input in self.portal_overlays.iter().copied() {
            if input.surface() != child.receipt.semantic_surface() {
                continue;
            }
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
            if input.surface() != node.receipt.semantic_surface() {
                return false;
            }
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

    /// Where this frame presents `instance`: Portal content through its
    /// Portal, every other occurrence where it is laid out.
    pub(super) fn portal_child_placement(
        &self,
        instance: UiMountedInstanceIdentity,
        surface: UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<UiMountedPlacement, UiMountedProjectionDenial> {
        self.portal_child_placement_with_work(instance, surface, binding)
            .map(|(placement, _, _)| placement)
    }

    /// Returns the placement, actual node-index probes, and overlay rows visited.
    pub(super) fn portal_child_placement_with_work(
        &self,
        instance: UiMountedInstanceIdentity,
        surface: UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<(UiMountedPlacement, usize, usize), UiMountedProjectionDenial> {
        let (node, mut probes) = self.semantic.nodes.get_with_probes(&instance);
        let Some(node) = node else {
            return Ok((UiMountedPlacement::InPlace, probes, 0));
        };
        let Some(owner_component) = node.portal_child_owner.as_ref() else {
            return Ok((UiMountedPlacement::InPlace, probes, 0));
        };
        let mut matched = None;
        for input in self.portal_overlays.iter().copied() {
            if input.surface() != surface {
                continue;
            }
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
            // Child occurrences share the owner's unpresented coordinate basis.
            // A nested Portal's observed anchor has already moved with its parent.
            let (source_surface, surface_work) = self.semantic.surface_for_with_probes(surface);
            probes = probes
                .checked_add(surface_work)
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
            let source_surface = source_surface
                .filter(|surface| surface.binding == binding)
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            let source_anchor = match super::surface_coordinates::viewport_allocation(
                *owner.occurrence_allocation.in_layout_space(),
                source_surface.coordinate_posture,
            )? {
                worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. }
                | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation { bounds, .. } => bounds,
                worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
                    return Err(UiMountedProjectionDenial::PortalOverlayOwnerMissing);
                }
            };
            if source_anchor.posture() != worth_ui_host_contract::UiMountedGeometryPosture::Area {
                return Err(UiMountedProjectionDenial::PortalOverlayCompletion(
                    worth_ui_host_contract::UiMountedPortalOverlayCompletionDenial::NonAreaGeometry,
                ));
            }
            if source_anchor.coordinate_space()
                != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
            {
                return Err(UiMountedProjectionDenial::PortalOverlayCompletion(
                    worth_ui_host_contract::UiMountedPortalOverlayCompletionDenial::CoordinateSpaceMismatch,
                ));
            }
            // Paint bounds begin where the content the Portal was fitted to
            // begins, which is not the owner's origin when that content is
            // laid out away from it.
            let content_anchor = input
                .placement()
                .content_anchor(
                    crate::mounting::presentation::UiPublishedRect::from_committed_box(
                        source_anchor,
                    ),
                )
                .canonical_box();
            matched = Some(UiPortalPresentation::fitted_to(
                input
                    .mechanic_for(self.frame, binding, receipt)
                    .map_err(UiMountedProjectionDenial::PortalOverlayCompletion)?,
                content_anchor,
            ));
        }
        Ok((
            matched.map_or(
                UiMountedPlacement::Hidden,
                UiMountedPlacement::ThroughPortal,
            ),
            probes,
            self.portal_overlays.len(),
        ))
    }
}
