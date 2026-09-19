use worth_ui_host_contract::{
    UiMountedAccessibilityProjection, UiMountedDiagnosticProjection, UiMountedOmissionReason,
    UiMountedParticipationStatus, UiMountedPresentationNodeChange,
    UiMountedPresentationNodeHitTest, UiMountedPresentationNodePaint,
    UiMountedPresentationNodeState, UiMountedPresentationNodeStateInput,
    UiMountedPreviewProjection,
};

use super::{UiMountedProjectionFrame, UiMountedProjectionNodeRecord, UiMountedProjectionSurface};

impl UiMountedProjectionFrame {
    pub(in crate::mounting) fn presentation_node_changes(
        &self,
        changed_instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Vec<UiMountedPresentationNodeChange> {
        let Some(surface) = self
            .semantic
            .surface_for(surface)
            .filter(|surface| surface.binding == binding)
        else {
            return Vec::new();
        };
        changed_instances
            .iter()
            .map(|instance| match self.semantic.nodes.get(instance) {
                Some(node) if node.receipt.semantic_surface() == surface.surface => match self
                    .portal_child_presentation(*instance, surface.surface, surface.binding)
                    .expect("prepared Portal children retain an unambiguous mounted owner")
                {
                    super::portal_child_view::UiMountedPortalChildPresentation::Ordinary => {
                        UiMountedPresentationNodeChange::Upsert(
                            self.presentation_node_state(node, surface, None),
                        )
                    }
                    super::portal_child_view::UiMountedPortalChildPresentation::Suppressed => {
                        UiMountedPresentationNodeChange::Remove(*instance)
                    }
                    super::portal_child_view::UiMountedPortalChildPresentation::Presented(
                        portal,
                        source_anchor,
                    ) => UiMountedPresentationNodeChange::Upsert(self.presentation_node_state(
                        node,
                        surface,
                        Some((portal, source_anchor)),
                    )),
                },
                Some(_) | None => UiMountedPresentationNodeChange::Remove(*instance),
            })
            .collect()
    }

    fn presentation_node_state(
        &self,
        node: &UiMountedProjectionNodeRecord,
        surface: UiMountedProjectionSurface,
        portal: Option<(
            worth_ui_host_contract::UiMountedPortalOverlayMechanic,
            worth_ui_host_contract::UiMountedCanonicalBox,
        )>,
    ) -> UiMountedPresentationNodeState {
        let receipt = &node.receipt;
        let audience = surface.audience;
        let occurrence = super::surface_coordinates::viewport_allocation(
            node.presentation_allocation(),
            surface.coordinate_posture,
        )
        .expect("completed occurrence geometry uses the admitted surface coordinates");
        let accessibility = if audience.accessibility_disclosed() {
            receipt.accessibility()
        } else {
            UiMountedAccessibilityProjection::Omitted(
                UiMountedOmissionReason::SurfacePolicyWithheld,
            )
        };
        UiMountedPresentationNodeState::from_runtime_mounting(UiMountedPresentationNodeStateInput {
            mounted_instance: receipt.mounted_instance(),
            authored_position: self
                .semantic
                .order
                .position(receipt.mounted_instance())
                .expect("a projected node remains in owner-authored order"),
            role: receipt.role(),
            participation: receipt.participation(),
            allocation: portal
                .map_or_else(
                    || Ok(occurrence),
                    |(portal, source_anchor)| {
                        super::super::appearance::portal_presented_allocation(
                            occurrence,
                            portal,
                            source_anchor,
                        )
                    },
                )
                .expect("completed occurrence allocation remains canonical in its Portal"),
            preview: self.presentation_preview(receipt.mounted_instance()),
            paint: self.presentation_node_paint(node),
            hit_test: self.presentation_node_hit_test(receipt.mounted_instance(), surface, portal),
            accessibility,
            motion: receipt.motion(),
            diagnostic: self.presentation_diagnostic(node, surface),
        })
    }

    fn presentation_node_hit_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface: UiMountedProjectionSurface,
        portal: Option<(
            worth_ui_host_contract::UiMountedPortalOverlayMechanic,
            worth_ui_host_contract::UiMountedCanonicalBox,
        )>,
    ) -> UiMountedPresentationNodeHitTest {
        self.mechanics
            .hit_test_for_instance(
                instance,
                surface.surface,
                surface.binding,
                self.frame,
                &self.receipt_basis,
            )
            .expect("prepared hit-test mechanics remain attributable")
            .and_then(|row| {
                portal.map_or(Some(row), |(portal, source_anchor)| {
                    row.presented_within_portal(portal, source_anchor)
                        .expect("validated Portal-relative hit region remains canonical")
                })
            })
            .map_or_else(
                || {
                    UiMountedPresentationNodeHitTest::Omitted(
                        UiMountedOmissionReason::NotProducedByExecutedLane,
                    )
                },
                UiMountedPresentationNodeHitTest::Region,
            )
    }

    fn presentation_preview(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> UiMountedPreviewProjection {
        self.preview
            .filter(|preview| preview.mounted_instance == instance)
            .map_or_else(
                || {
                    UiMountedPreviewProjection::Omitted(
                        UiMountedOmissionReason::NotProducedByExecutedLane,
                    )
                },
                |preview| {
                    UiMountedPreviewProjection::resize(
                        preview.frame_epoch,
                        preview.extent_subpixels,
                        preview.candidate_count,
                        preview.all_candidates_admitted,
                    )
                },
            )
    }

    fn presentation_diagnostic(
        &self,
        node: &UiMountedProjectionNodeRecord,
        surface: UiMountedProjectionSurface,
    ) -> UiMountedDiagnosticProjection {
        if !surface.audience.diagnostics_disclosed() {
            return UiMountedDiagnosticProjection::Omitted(
                UiMountedOmissionReason::SurfacePolicyWithheld,
            );
        }
        self.visual_overlay
            .filter(|overlay| {
                overlay.target_receipt.mounted_instance() == node.receipt.mounted_instance()
            })
            .and_then(|overlay| overlay.mechanic_for(self.frame, surface.surface, surface.binding))
            .map_or_else(
                || node.receipt.diagnostic(),
                UiMountedDiagnosticProjection::IdentityOverlay,
            )
    }

    fn presentation_node_paint(
        &self,
        node: &UiMountedProjectionNodeRecord,
    ) -> UiMountedPresentationNodePaint {
        if node.receipt.participation().paint().status() != UiMountedParticipationStatus::Admitted {
            return UiMountedPresentationNodePaint::Omitted(
                UiMountedOmissionReason::NotProducedByExecutedLane,
            );
        }
        self.plan_index_paint_selectors
            .iter()
            .rev()
            .find_map(|selector| selector.batch_for(node.plan_index))
            .or_else(|| {
                self.ordinary_paint_selector
                    .as_ref()
                    .and_then(|selector| selector.batch_for(node.plan_index))
            })
            .map_or_else(
                || {
                    UiMountedPresentationNodePaint::Omitted(
                        UiMountedOmissionReason::NotProducedByExecutedLane,
                    )
                },
                |batch| UiMountedPresentationNodePaint::CountOnlyBatch(batch.index()),
            )
    }
}
