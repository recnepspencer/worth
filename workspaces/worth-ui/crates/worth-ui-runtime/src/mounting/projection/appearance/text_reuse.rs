use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterPinRequest, UiMountedSemanticTextMechanic,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedTextForegroundReuseProof {
    layout_reused: bool,
    geometry_reused: bool,
    raster_reused: bool,
    atlas_pin_requests_reused: bool,
}

impl UiMountedTextForegroundReuseProof {
    pub(super) const fn layout_reused(self) -> bool {
        self.layout_reused
    }

    pub(super) const fn geometry_reused(self) -> bool {
        self.geometry_reused
    }

    pub(super) const fn raster_reused(self) -> bool {
        self.raster_reused
    }

    pub(super) const fn atlas_pin_requests_reused(self) -> bool {
        self.atlas_pin_requests_reused
    }

    pub(super) const fn paint_only(self) -> bool {
        self.layout_reused
            && self.geometry_reused
            && self.raster_reused
            && self.atlas_pin_requests_reused
    }
}

pub(crate) struct UiMountedAppearanceTextReuseInput<'left, 'right> {
    pub(super) predecessor: &'left UiMountedSemanticTextMechanic,
    pub(super) successor: &'right UiMountedSemanticTextMechanic,
    pub(super) predecessor_demand: UiGlyphRasterDemandBatchView<'left>,
    pub(super) successor_demand: UiGlyphRasterDemandBatchView<'right>,
    pub(super) predecessor_pins: &'left [UiGlyphRasterPinRequest],
    pub(super) successor_pins: &'right [UiGlyphRasterPinRequest],
}

impl<'left, 'right> UiMountedAppearanceTextReuseInput<'left, 'right> {
    pub(crate) const fn new(
        predecessor: &'left UiMountedSemanticTextMechanic,
        successor: &'right UiMountedSemanticTextMechanic,
        predecessor_demand: UiGlyphRasterDemandBatchView<'left>,
        successor_demand: UiGlyphRasterDemandBatchView<'right>,
        predecessor_pins: &'left [UiGlyphRasterPinRequest],
        successor_pins: &'right [UiGlyphRasterPinRequest],
    ) -> Self {
        Self {
            predecessor,
            successor,
            predecessor_demand,
            successor_demand,
            predecessor_pins,
            successor_pins,
        }
    }
}

pub(super) fn prove(
    input: UiMountedAppearanceTextReuseInput<'_, '_>,
) -> UiMountedTextForegroundReuseProof {
    let layout_reused = same_layout(input.predecessor, input.successor)
        && input.successor.performed_layout_cost().is_none();
    let geometry_reused = same_geometry(input.predecessor, input.successor)
        && same_foreground_ranges(input.predecessor, input.successor);
    let raster_reused = layout_reused
        && geometry_reused
        && same_demand(input.predecessor_demand, input.successor_demand);
    let atlas_pin_requests_reused = raster_reused && input.predecessor_pins == input.successor_pins;
    UiMountedTextForegroundReuseProof {
        layout_reused,
        geometry_reused,
        raster_reused,
        atlas_pin_requests_reused,
    }
}

fn same_layout(
    predecessor: &UiMountedSemanticTextMechanic,
    successor: &UiMountedSemanticTextMechanic,
) -> bool {
    predecessor.surface() == successor.surface()
        && predecessor.text() == successor.text()
        && predecessor.qualified_layout_identity() == successor.qualified_layout_identity()
        && predecessor.qualified_layout_request() == successor.qualified_layout_request()
        && predecessor.qualified_layout_profile() == successor.qualified_layout_profile()
        && predecessor.qualified_layout_fonts() == successor.qualified_layout_fonts()
        && predecessor.qualified_layout_scale() == successor.qualified_layout_scale()
        && predecessor.qualified_layout_width() == successor.qualified_layout_width()
        && predecessor.slot() == successor.slot()
        && predecessor.collection_row() == successor.collection_row()
        && predecessor.profile() == successor.profile()
        && predecessor.layer_semantic_order() == successor.layer_semantic_order()
}

fn same_geometry(
    predecessor: &UiMountedSemanticTextMechanic,
    successor: &UiMountedSemanticTextMechanic,
) -> bool {
    predecessor.allocation_basis() == successor.allocation_basis()
        && predecessor.bounds() == successor.bounds()
        && predecessor.clip_bounds() == successor.clip_bounds()
        && predecessor.origin_x() == successor.origin_x()
        && predecessor.origin_y() == successor.origin_y()
}

fn same_foreground_ranges(
    predecessor: &UiMountedSemanticTextMechanic,
    successor: &UiMountedSemanticTextMechanic,
) -> bool {
    predecessor.foregrounds().len() == successor.foregrounds().len()
        && predecessor
            .foregrounds()
            .iter()
            .zip(successor.foregrounds())
            .all(|(left, right)| {
                left.original_range() == right.original_range()
                    && left.identity() == right.identity()
            })
}

fn same_demand(
    predecessor: UiGlyphRasterDemandBatchView<'_>,
    successor: UiGlyphRasterDemandBatchView<'_>,
) -> bool {
    predecessor.identity() == successor.identity()
        && predecessor.layout_identity() == successor.layout_identity()
        && predecessor.dpi_milli() == successor.dpi_milli()
        && predecessor.text_scale_generation() == successor.text_scale_generation()
        && predecessor.lane() == successor.lane()
        && predecessor.records().len() == successor.records().len()
        && predecessor
            .records()
            .iter()
            .zip(successor.records())
            .all(|(left, right)| {
                left.key() == right.key()
                    && left.attribution() == right.attribution()
                    && left.extent() == right.extent()
                    && left.staged_bytes() == right.staged_bytes()
            })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::certification_support::{
        initial_presentation_mechanics_for_certification,
        semantic_text_projection_for_certification, UiSemanticTextProjectionCertificationMutation,
    };
    use crate::mounting::qualified_text_test_support::inert_qualified_layout;
    use crate::native_platform::text_presentation::{
        prepare_mounted_semantic_text, UiMountedEventTimeDpiAuthority,
        UiNativeTextPresentationPreparation,
    };
    use worth_ui_host_contract::{
        UiGlyphRasterDemandIdentity, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
        UiMountedPaintCommand, UiMountedPresentationWorkView, UiMountedRgba8,
        UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic,
        UiMountedSurfaceBindingRequirement, UiMountedTextForegroundSpan,
        WorthUiHostCapabilityObservationGeneration,
    };

    fn reused_successor(
        predecessor: &UiMountedSemanticTextMechanic,
        layout: &worth_ui_text::UiQualifiedTextLayout,
    ) -> UiMountedSemanticTextMechanic {
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let foregrounds = predecessor
            .foregrounds()
            .iter()
            .map(|span| {
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    span.original_range(),
                    UiMountedRgba8::new(247, 129, 47, 255),
                    span.identity(),
                )
            })
            .collect::<Vec<_>>();
        UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
            UiMountedSemanticTextCompletionInput {
                content_generation:
                    worth_ui_host_contract::UiMountedContentGeneration::mint_unbound().unwrap(),
                frame,
                surface: predecessor.surface(),
                binding: predecessor.binding(),
                mounted_instance: predecessor.mounted_instance(),
                node_receipt: issuer.receipt_for(predecessor.mounted_instance()),
                allocation_basis: predecessor.allocation_basis(),
                bounds: predecessor.bounds(),
                clip_bounds: predecessor.clip_bounds(),
                origin_x: predecessor.origin_x(),
                origin_y: predecessor.origin_y(),
                text: Arc::from(predecessor.text()),
                layout: layout.view(),
                slot: predecessor.slot(),
                collection_row: predecessor.collection_row().cloned(),
                foregrounds: Arc::from(foregrounds),
                profile: predecessor.profile(),
                layer_semantic_order: predecessor.layer_semantic_order(),
                capability_generation: predecessor.capability_generation(),
                capability_profile_digest: predecessor.capability_profile_digest().wrapping_add(1),
            },
        )
        .unwrap()
    }

    #[test]
    fn exact_qualified_text_boundary_proves_layout_geometry_raster_and_pin_reuse() {
        let projection = semantic_text_projection_for_certification(
            UiSemanticTextProjectionCertificationMutation::Exact,
        );
        let mechanic_requirement = UiMountedSurfaceBindingRequirement::new(
            projection.surface(),
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            projection.binding(),
            WorthUiHostCapabilityObservationGeneration::new(7),
            1_000,
            UiHostSurfacePresentationMode::NativeDisplay,
        );
        let initial =
            initial_presentation_mechanics_for_certification(&projection, mechanic_requirement);
        let text = initial
            .commands()
            .iter()
            .find_map(|command| match command {
                UiMountedPaintCommand::SemanticText { mechanic, .. } => Some(mechanic),
                _ => None,
            })
            .expect("certification projection contains semantic text");
        let layout = inert_qualified_layout(text.text());
        let UiNativeTextPresentationPreparation::Prepared(prepared) =
            prepare_mounted_semantic_text(
                UiMountedPresentationWorkView::Initial(&initial),
                UiMountedEventTimeDpiAuthority::from_requirement(mechanic_requirement).unwrap(),
                |identity| (identity == layout.identity()).then_some(layout.as_ref()),
            )
            .unwrap()
        else {
            panic!("exact text preparation must be admitted");
        };
        let successor = reused_successor(text, layout.as_ref());
        let demand = prepared.demand_batches()[0].as_view();
        let pins = prepared.demand_batches()[0]
            .records()
            .iter()
            .map(|record| {
                UiGlyphRasterPinRequest::from_text_mechanics(demand.layout_identity(), record.key())
            })
            .collect::<Vec<_>>();
        let proof = prove(UiMountedAppearanceTextReuseInput::new(
            text, &successor, demand, demand, &pins, &pins,
        ));
        assert!(proof.paint_only());
        assert!(proof.layout_reused());
        assert!(proof.geometry_reused());
        assert!(proof.raster_reused());
        assert!(proof.atlas_pin_requests_reused());
    }

    #[test]
    fn a_foreign_demand_identity_denies_raster_and_paint_only_reuse() {
        let projection = semantic_text_projection_for_certification(
            UiSemanticTextProjectionCertificationMutation::Exact,
        );
        let requirement = UiMountedSurfaceBindingRequirement::new(
            projection.surface(),
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            projection.binding(),
            WorthUiHostCapabilityObservationGeneration::new(7),
            1_000,
            UiHostSurfacePresentationMode::NativeDisplay,
        );
        let initial = initial_presentation_mechanics_for_certification(&projection, requirement);
        let text = initial
            .commands()
            .iter()
            .find_map(|command| match command {
                UiMountedPaintCommand::SemanticText { mechanic, .. } => Some(mechanic),
                _ => None,
            })
            .unwrap();
        let layout = inert_qualified_layout(text.text());
        let UiNativeTextPresentationPreparation::Prepared(prepared) =
            prepare_mounted_semantic_text(
                UiMountedPresentationWorkView::Initial(&initial),
                UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap(),
                |identity| (identity == layout.identity()).then_some(layout.as_ref()),
            )
            .unwrap()
        else {
            panic!("exact text preparation must be admitted");
        };
        let successor = reused_successor(text, layout.as_ref());
        let demand = prepared.demand_batches()[0].as_view();
        let foreign = UiGlyphRasterDemandBatchView::from_text_mechanics(
            worth_ui_host_contract::UiGlyphRasterDemandBatchViewInput {
                identity: UiGlyphRasterDemandIdentity::from_text_mechanics([9; 32]),
                layout: demand.layout_identity(),
                dpi_milli: demand.dpi_milli(),
                text_scale: demand.text_scale_generation(),
                lane: demand.lane(),
                records: demand.records(),
            },
        )
        .unwrap();
        let proof = prove(UiMountedAppearanceTextReuseInput::new(
            text,
            &successor,
            demand,
            foreign,
            &[],
            &[],
        ));
        assert!(proof.layout_reused());
        assert!(proof.geometry_reused());
        assert!(!proof.raster_reused());
        assert!(!proof.paint_only());
    }
}
