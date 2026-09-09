use worth_ui_host_contract::{
    UiMountedAccessibilityProjection, UiMountedClipTable, UiMountedDiagnosticProjection,
    UiMountedFilledRectTable, UiMountedHitTestProjection, UiMountedHitTestTable,
    UiMountedLayerTable, UiMountedNodeProjectionView, UiMountedNodeProjectionViewInput,
    UiMountedOmissionReason, UiMountedPaintBatchReference, UiMountedPaintBatchTable,
    UiMountedPaintProjection, UiMountedParticipationStatus, UiMountedPortalOverlayTable,
    UiMountedPreviewProjection, UiMountedProjectionAudience, UiMountedProjectionView,
    UiMountedProjectionViewInput, UiMountedRealtimeBatchTable, UiMountedResourceTable,
    UiMountedSemanticTextTable, UiMountedSpatialBatchTable, UiSurfaceBindingGeneration,
};

use super::super::{UiMountedNodeReceipt, UiMountedProjectionDenial};
use super::{UiMountedProjectionFrame, UiMountedProjectionNodeRecord, UiMountedProjectionSurface};

use super::drawable_order::{
    drawable_reference_index, validate_drawable_coverage, UiMountedDrawableReferenceIndex,
};
use super::portal_mechanic_view::{
    portal_relative_allocation, UiMountedFilledRectReferenceIndex, UiMountedHitTestReferenceIndex,
};

struct UiMountedNodeViewContext<'a> {
    surface: UiMountedProjectionSurface,
    filled_rect_by_instance: &'a UiMountedFilledRectReferenceIndex,
    hit_test_by_instance: &'a UiMountedHitTestReferenceIndex,
    semantic_text_by_instance: &'a super::semantic_text_view::UiMountedSemanticTextReferenceIndex,
    drawables_by_instance: &'a UiMountedDrawableReferenceIndex,
}

#[derive(Clone)]
pub(super) struct UiMountedOrdinaryPaintSelector {
    receipt: crate::runtime::WorthUiOrdinaryLaneFrameReceipt,
    batch: UiMountedPaintBatchReference,
}

#[derive(Clone)]
pub(super) struct UiMountedPlanIndexPaintSelector {
    indexes: Box<[u32]>,
    batch: UiMountedPaintBatchReference,
}

impl UiMountedProjectionFrame {
    pub(in crate::mounting) const fn hit_index_maintenance_work(
        &self,
    ) -> crate::mounting::hit_test_work::UiHitTestSpatialWork {
        self.hit_index_work
    }

    pub fn view_for(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<UiMountedProjectionView, UiMountedProjectionDenial> {
        let surface = self
            .semantic
            .surfaces
            .get(&binding)
            .copied()
            .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
        let filled_rects = self.filled_rect_view_rows(surface)?;
        let hit_tests = self.hit_test_view_rows(surface)?;
        let semantic_text = self.semantic_text_view_rows(surface)?;
        let portal_overlays = self.portal_overlay_view_rows(surface)?;
        let drawables = drawable_reference_index(
            &filled_rects.rows,
            &portal_overlays.rows,
            &semantic_text.rows,
        )?;
        let node_view_context = UiMountedNodeViewContext {
            surface,
            filled_rect_by_instance: &filled_rects.references,
            hit_test_by_instance: &hit_tests.references,
            semantic_text_by_instance: &semantic_text.references,
            drawables_by_instance: &drawables,
        };
        let mut nodes = Vec::new();
        for node in self
            .semantic
            .order
            .iter()
            .filter_map(|instance| self.semantic.nodes.get(instance))
            .filter(|node| node.receipt.semantic_surface() == surface.surface)
        {
            let (allocation, portal_presentation) = match self.portal_child_presentation(
                node.receipt.mounted_instance(),
                surface.surface,
                surface.binding,
            )? {
                super::portal_child_view::UiMountedPortalChildPresentation::Ordinary => {
                    (None, None)
                }
                super::portal_child_view::UiMountedPortalChildPresentation::Suppressed => continue,
                super::portal_child_view::UiMountedPortalChildPresentation::Presented(portal) => {
                    (
                        Some(portal_relative_allocation(
                            node.presentation_allocation(),
                            portal,
                        )?),
                        Some(
                            worth_ui_host_contract::UiMountedPortalPresentationAffinity::from_runtime_mounting(
                                portal.owner(),
                                portal.portal_identity(),
                            ),
                        ),
                    )
                }
            };
            nodes.push(self.audience_node_view(
                node,
                &node_view_context,
                allocation,
                portal_presentation,
            ));
        }
        validate_drawable_coverage(
            &drawables,
            &nodes,
            filled_rects.rows.len() + portal_overlays.rows.len() + semantic_text.rows.len(),
        )?;
        let materialized_rows = [
            nodes.len(),
            filled_rects.rows.len(),
            semantic_text.rows.len(),
            portal_overlays.rows.len(),
            hit_tests.rows.len(),
            self.layers.len(),
            self.paint_batches.len(),
            self.spatial_batches.len(),
            self.realtime_batches.len(),
            self.resources.len(),
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
        .and_then(|rows| u64::try_from(rows).ok())
        .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        let (authored_paint_commands, authored_paint_order) = super::presentation_sources::compile(
            &nodes,
            &filled_rects.rows,
            &portal_overlays.rows,
            &semantic_text.rows,
        );
        let filled_rects = UiMountedFilledRectTable::from_runtime_mounting(filled_rects.rows)
            .map_err(|_| UiMountedProjectionDenial::StaticPaintCapacityExceeded)?;
        let hit_tests = UiMountedHitTestTable::from_runtime_mounting(hit_tests.rows)
            .ok_or(UiMountedProjectionDenial::HitTestCapacityExceeded)?;
        let semantic_text = UiMountedSemanticTextTable::from_runtime_mounting(semantic_text.rows)
            .map_err(|_| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
        let portal_overlays =
            UiMountedPortalOverlayTable::from_runtime_mounting(portal_overlays.rows)
                .ok_or(UiMountedProjectionDenial::PortalOverlayCapacityExceeded)?;
        let projection = UiMountedProjectionView::new(UiMountedProjectionViewInput {
            frame: self.frame,
            surface: surface.surface,
            binding,
            content_generation: self.content_generation,
            nodes,
            clips: UiMountedClipTable::produced(Vec::new()),
            layers: UiMountedLayerTable::produced(self.layers.clone()),
            filled_rects,
            portal_overlays,
            semantic_text,
            hit_tests,
            paint_batches: UiMountedPaintBatchTable::new(self.paint_batches.clone()),
            spatial_batches: UiMountedSpatialBatchTable::new(self.spatial_batches.clone()),
            realtime_batches: UiMountedRealtimeBatchTable::new(self.realtime_batches.clone()),
            resources: UiMountedResourceTable::new(self.resources.clone()),
            authored_paint_commands,
            authored_paint_order,
        });
        self.materialized_projection_rows.set(
            self.materialized_projection_rows
                .get()
                .checked_add(materialized_rows)
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?,
        );
        Ok(projection)
    }

    fn audience_node_view(
        &self,
        node: &UiMountedProjectionNodeRecord,
        context: &UiMountedNodeViewContext<'_>,
        allocation: Option<worth_ui_host_contract::UiMountedAllocationProjection>,
        portal_presentation: Option<worth_ui_host_contract::UiMountedPortalPresentationAffinity>,
    ) -> UiMountedNodeProjectionView {
        let receipt = &node.receipt;
        let audience = context.surface.audience;
        let accessibility = if audience.accessibility_disclosed() {
            receipt.accessibility()
        } else {
            UiMountedAccessibilityProjection::Omitted(
                UiMountedOmissionReason::SurfacePolicyWithheld,
            )
        };
        let diagnostic = self.diagnostic_for(receipt, context.surface, audience);
        UiMountedNodeProjectionView::new(UiMountedNodeProjectionViewInput {
            mounted_instance: receipt.mounted_instance(),
            node_receipt: self
                .receipt_basis
                .receipt_for(receipt.mounted_instance())
                .expect("projected semantic nodes belong to the frame receipt basis"),
            authored_position: self
                .semantic
                .order
                .position(receipt.mounted_instance())
                .expect("a projected node remains in owner-authored order"),
            role: receipt.role(),
            participation: receipt.participation(),
            allocation: allocation.unwrap_or_else(|| receipt.allocation()),
            preview: self.preview_for(receipt),
            paint: self.paint_for(node, context.filled_rect_by_instance),
            hit_test: context
                .hit_test_by_instance
                .get(&receipt.mounted_instance())
                .copied()
                .map_or_else(
                    || {
                        UiMountedHitTestProjection::Omitted(
                            UiMountedOmissionReason::NotProducedByExecutedLane,
                        )
                    },
                    UiMountedHitTestProjection::Region,
                ),
            accessibility,
            motion: receipt.motion(),
            diagnostic,
            drawables: context
                .drawables_by_instance
                .get(&receipt.mounted_instance())
                .map(|drawables| drawables.to_vec())
                .unwrap_or_default(),
            semantic_text: context
                .semantic_text_by_instance
                .get(&receipt.mounted_instance())
                .cloned()
                .unwrap_or_default(),
            portal_presentation,
        })
    }

    fn diagnostic_for(
        &self,
        receipt: &UiMountedNodeReceipt,
        surface: UiMountedProjectionSurface,
        audience: UiMountedProjectionAudience,
    ) -> UiMountedDiagnosticProjection {
        if !audience.diagnostics_disclosed() {
            return UiMountedDiagnosticProjection::Omitted(
                UiMountedOmissionReason::SurfacePolicyWithheld,
            );
        }
        self.visual_overlay
            .filter(|overlay| {
                overlay.target_receipt.mounted_instance() == receipt.mounted_instance()
            })
            .and_then(|overlay| overlay.mechanic_for(self.frame, surface.surface, surface.binding))
            .map_or_else(
                || receipt.diagnostic(),
                UiMountedDiagnosticProjection::IdentityOverlay,
            )
    }

    fn paint_for(
        &self,
        node: &UiMountedProjectionNodeRecord,
        filled_rect_by_instance: &UiMountedFilledRectReferenceIndex,
    ) -> UiMountedPaintProjection {
        if node.receipt.participation().paint().status() != UiMountedParticipationStatus::Admitted {
            return UiMountedPaintProjection::Omitted(
                UiMountedOmissionReason::NotProducedByExecutedLane,
            );
        }
        if let Some(reference) = filled_rect_by_instance.get(&node.receipt.mounted_instance()) {
            return UiMountedPaintProjection::FilledRect(*reference);
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
                    UiMountedPaintProjection::Omitted(
                        UiMountedOmissionReason::NotProducedByExecutedLane,
                    )
                },
                UiMountedPaintProjection::CountOnlyBatch,
            )
    }

    fn preview_for(&self, receipt: &UiMountedNodeReceipt) -> UiMountedPreviewProjection {
        self.preview
            .filter(|preview| preview.mounted_instance == receipt.mounted_instance())
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
}

impl UiMountedOrdinaryPaintSelector {
    pub(super) fn new(
        receipt: crate::runtime::WorthUiOrdinaryLaneFrameReceipt,
        batch: UiMountedPaintBatchReference,
    ) -> Self {
        Self { receipt, batch }
    }

    pub(super) fn batch_for(
        &self,
        plan_index: Option<u32>,
    ) -> Option<UiMountedPaintBatchReference> {
        let plan_index = plan_index?;
        self.receipt
            .touch()
            .names_plan_index(plan_index)
            .then_some(self.batch)
    }
}

impl UiMountedPlanIndexPaintSelector {
    pub(super) fn new(indexes: Box<[u32]>, batch: UiMountedPaintBatchReference) -> Self {
        Self { indexes, batch }
    }

    pub(super) fn batch_for(
        &self,
        plan_index: Option<u32>,
    ) -> Option<UiMountedPaintBatchReference> {
        let plan_index = plan_index?;
        self.indexes.contains(&plan_index).then_some(self.batch)
    }
}
