use super::{
    prepare_mounted_semantic_text, UiMountedEventTimeDpiAuthority,
    UiNativeTextPresentationPreparation, UiNativeTextPresentationReadiness,
};
use crate::certification_support::{
    initial_presentation_mechanics_for_certification, semantic_text_projection_for_certification,
    UiSemanticTextProjectionCertificationMutation,
};
use crate::mounting::qualified_text_test_support::inert_qualified_layout;
use worth_ui_host_contract::{
    UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPaintOrderIntegrity, UiMountedPresentationDelta,
    UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationInitialInput, UiMountedPresentationWorkView, UiMountedRgba8,
    UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic,
    UiMountedSurfaceBindingRequirement, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiTextOriginalRange,
    WorthUiHostCapabilityObservationGeneration,
};

fn requirement(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> UiMountedSurfaceBindingRequirement {
    UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    )
}

#[test]
fn demand_preparation_stops_at_typed_atlas_plan_boundary_without_raster_work() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement(&projection);
    let initial = initial_presentation_mechanics_for_certification(&projection, requirement);
    let layout = inert_qualified_layout("ONLINE");
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap();
    let preparation = prepare_mounted_semantic_text(
        UiMountedPresentationWorkView::Initial(&initial),
        dpi,
        |_| Some(layout.as_ref()),
    )
    .unwrap();
    let UiNativeTextPresentationPreparation::Prepared(prepared) = preparation else {
        panic!("exact mounted text must prepare a native transaction");
    };
    assert_eq!(prepared.layout_count(), 1);
    assert_eq!(prepared.demand_batches().len(), 1);
    let planning = prepared.planning_inspection().unwrap();
    assert_eq!(planning.demand_batches(), 1);
    assert_eq!(
        usize::try_from(planning.demand_records()).unwrap(),
        prepared.demand_batches()[0].records().len()
    );
    assert_eq!(planning.key_checks(), planning.demand_records());
    assert_eq!(prepared.raster_work().rasterized_glyphs(), 0);
    assert_eq!(prepared.raster_work().rasterized_texels(), 0);
    assert_eq!(prepared.raster_work().produced_bytes(), 0);
    assert_eq!(prepared.paint_span_count(), 1);
}

#[test]
fn consumer_layout_substitution_is_rejected_before_layout_admission() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement(&projection);
    let initial = initial_presentation_mechanics_for_certification(&projection, requirement);
    let substituted = inert_qualified_layout("SYSTEM FONT SUBSTITUTE");
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap();
    let preparation = prepare_mounted_semantic_text(
        UiMountedPresentationWorkView::Initial(&initial),
        dpi,
        |_| Some(substituted.as_ref()),
    )
    .unwrap();
    let UiNativeTextPresentationPreparation::Denied(denial) = preparation else {
        panic!("a substituted layout must be denied before preparation");
    };
    assert_eq!(
        denial.readiness(),
        UiNativeTextPresentationReadiness::LayoutMismatch
    );
}

#[test]
fn removal_only_delta_preserves_explicit_command_identity_without_raster_demand() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement(&projection);
    let initial = initial_presentation_mechanics_for_certification(&projection, requirement);
    let removed = initial.commands()[0].identity();
    let affinity = initial.affinity();
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: affinity.successor(),
        successor: worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: affinity.surface(),
        binding: affinity.binding(),
        content: affinity.content(),
        baseline: affinity.baseline(),
        changes: vec![UiMountedPaintCommandChange::Remove(removed)],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[]),
        damage: vec![
            worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting(
                initial.commands()[0].bounds(),
            ),
        ],
        auxiliary: None,
        production_cost: initial.production_cost(),
    });
    let preparation = prepare_mounted_semantic_text(
        UiMountedPresentationWorkView::Delta(&delta),
        UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap(),
        |_| None,
    )
    .expect("an explicit removal must reach the committed text-pin owner");
    let UiNativeTextPresentationPreparation::Prepared(prepared) = preparation else {
        panic!("removal-only work cannot require layout or raster admission");
    };
    assert!(prepared.demand_batches().is_empty());
    assert!(prepared.pin_commands().is_empty());
    assert_eq!(prepared.pin_removals(), &[removed]);
}

#[test]
fn complete_empty_text_set_reaches_the_committed_pin_owner() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement(&projection);
    let populated = initial_presentation_mechanics_for_certification(&projection, requirement);
    let affinity = populated.affinity();
    let empty =
        UiMountedPresentationInitial::from_inert_mechanics(UiMountedPresentationInitialInput {
            successor: affinity.successor(),
            surface: affinity.surface(),
            binding: affinity.binding(),
            content: affinity.content(),
            baseline: affinity.baseline(),
            projection,
            commands: Vec::new(),
            order: Vec::new(),
            order_integrity: UiMountedPaintOrderIntegrity::for_order(&[]),
            damage: Vec::new(),
            production_cost: Default::default(),
        });
    let preparation = prepare_mounted_semantic_text(
        UiMountedPresentationWorkView::Initial(&empty),
        UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap(),
        |_| None,
    )
    .expect("an empty complete set must reach the committed text-pin owner");
    let UiNativeTextPresentationPreparation::Prepared(prepared) = preparation else {
        panic!("an empty complete set performs no layout or raster admission");
    };
    assert!(prepared.pin_set_complete());
    assert!(prepared.demand_batches().is_empty());
    assert!(prepared.pin_commands().is_empty());
    assert!(prepared.pin_removals().is_empty());
}

#[test]
fn mixed_bidi_native_runs_keep_logical_paint_ownership() {
    crate::mounting::prove_paint_only_mechanic_locality();
    let (layout, mechanic, command, damage) = mixed_bidi_paint_world();
    let join = super::MountedTextDemandJoin {
        dpi: super::UiMountedEventTimeDpiAuthority(std::num::NonZeroU32::new(1_000).unwrap()),
        lane: worth_ui_host_contract::UiGlyphRasterLane::Ordinary,
        selection: worth_ui_text::UiGlyphRasterDemandSelection::LogicalDamage(&damage),
        resolve: |_| Some(layout.as_ref()),
        _layout: std::marker::PhantomData,
    };
    let prepared = super::prepare_demands(&[(command, &mechanic)], &join).unwrap();
    let blue_repaint = worth_ui_text::derive_glyph_raster_demand(
        &layout,
        worth_ui_text::UiGlyphRasterDemandRequest {
            paint_spans: &[
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    UiTextOriginalRange::new(0, 4).unwrap(),
                    UiMountedRgba8::new(12, 34, 56, 255),
                    UiMountedTextPaintSpanIdentity::from_runtime_mounting([17; 32]),
                ),
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    UiTextOriginalRange::new(4, 10).unwrap(),
                    UiMountedRgba8::new(78, 90, 123, 255),
                    UiMountedTextPaintSpanIdentity::from_runtime_mounting([29; 32]),
                ),
            ],
            selection: worth_ui_text::UiGlyphRasterDemandSelection::LogicalDamage(&damage),
            scale: worth_ui_text::UiGlyphRasterScale::new(1_000, mechanic.qualified_layout_scale())
                .unwrap(),
            placement: worth_ui_text::UiGlyphRasterPlacement::from_mounted_logical(
                mechanic.origin_x(),
                mechanic.origin_y(),
            )
            .unwrap(),
            lane: worth_ui_host_contract::UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap();
    assert!(prepared.demands[0]
        .records()
        .iter()
        .zip(blue_repaint.records())
        .all(|(before, after)| before.key() == after.key()));
    let first = UiTextOriginalRange::new(0, 4).unwrap();
    let second = UiTextOriginalRange::new(4, 10).unwrap();
    let red = UiMountedRgba8::new(220, 20, 60, 255);
    let blue = UiMountedRgba8::new(30, 144, 255, 255);
    let mut observed_first = 0;
    let mut observed_second = 0;
    for run in prepared.glyph_runs.iter().copied() {
        let (owner, color, paint_digest) = if range_contains(first, run.original_range()) {
            observed_first += 1;
            (first, red, [17; 32])
        } else {
            observed_second += 1;
            (second, blue, [29; 32])
        };
        assert!(range_contains(owner, run.original_range()));
        assert_eq!(run.foreground(), color);
        assert_eq!(run.paint_span().digest(), paint_digest);
        assert_eq!(run.layout_identity(), layout.identity());
    }
    assert!(observed_first > 0 && observed_second > 0);
    assert!(layout
        .visual_runs()
        .iter()
        .any(|run| !run.bidi_level().is_multiple_of(2)));
}

#[test]
fn single_color_and_logical_order_mutants_disagree_with_native_runs() {
    let (layout, mechanic, command, damage) = mixed_bidi_paint_world();
    let join = super::MountedTextDemandJoin {
        dpi: super::UiMountedEventTimeDpiAuthority(std::num::NonZeroU32::new(1_000).unwrap()),
        lane: worth_ui_host_contract::UiGlyphRasterLane::Ordinary,
        selection: worth_ui_text::UiGlyphRasterDemandSelection::LogicalDamage(&damage),
        resolve: |_| Some(layout.as_ref()),
        _layout: std::marker::PhantomData,
    };
    let prepared = super::prepare_demands(&[(command, &mechanic)], &join).unwrap();
    let observed = prepared
        .glyph_runs
        .iter()
        .map(|run| (run.original_range(), run.foreground()))
        .collect::<Vec<_>>();
    let single_color = observed
        .iter()
        .map(|(range, _)| (*range, UiMountedRgba8::new(220, 20, 60, 255)))
        .collect::<Vec<_>>();
    let mut logical_order = observed.clone();
    logical_order.sort_by_key(|(range, _)| range.start());
    assert_ne!(observed, single_color);
    assert_ne!(observed, logical_order);
}

fn mixed_bidi_paint_world() -> (
    std::sync::Arc<worth_ui_text::UiQualifiedTextLayout>,
    UiMountedSemanticTextMechanic,
    UiMountedPaintCommandIdentity,
    [worth_ui_host_contract::UiMountedLogicalDamage; 1],
) {
    let source: std::sync::Arc<str> = std::sync::Arc::from("abc \u{05d0}\u{05d1}\u{05d2}");
    let first = UiTextOriginalRange::new(0, 4).unwrap();
    let second = UiTextOriginalRange::new(4, 10).unwrap();
    let constraints = worth_ui_text::UiTextParagraphConstraints::new(
        worth_ui_text::UiTextParagraphConstraintsInput {
            language: std::sync::Arc::from("und"),
            base_direction: worth_ui_text::UiTextBaseDirection::Auto,
            wrap: worth_ui_text::UiTextWrap::UnicodeWord,
            alignment: worth_ui_text::UiTextAlignment::Start,
            overflow: worth_ui_text::UiTextOverflow::Clip,
            font_size_millipoints: 14_000,
            width_millipoints: 160_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 1,
        },
    )
    .unwrap();
    let style = worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints);
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = std::sync::Arc::new(fonts);
    let layout = std::sync::Arc::new(
        worth_ui_text::qualify_text_layout(
            worth_ui_text::UiTextParagraphAdmissionInput {
                source: std::sync::Arc::clone(&source),
                constraints,
                profile_generation: worth_ui_host_contract::UiTextProfileGeneration::new(1)
                    .unwrap(),
                font_collection_generation: fonts.generation(),
                text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1)
                    .unwrap(),
                styles: Box::new([
                    worth_ui_text::UiTextStyleSpan::new(first, style.clone()).unwrap(),
                    worth_ui_text::UiTextStyleSpan::new(second, style).unwrap(),
                ]),
            },
            fonts,
        )
        .unwrap(),
    );
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 48.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .unwrap();
    let mechanic = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
                .unwrap(),
            frame,
            surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            mounted_instance: instance,
            node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame)
                .unwrap()
                .receipt_for(instance),
            allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis::new(
                1,
                1,
                1,
                worth_ui_host_contract::UiMountedTransformProjection::Identity,
            ),
            bounds,
            clip_bounds: bounds,
            origin_x: 0.0,
            origin_y: 0.0,
            text: source,
            layout: layout.view(),
            slot: worth_ui_host_contract::UiSemanticTextSlot::Value,
            collection_row: None,
            foregrounds: std::sync::Arc::from([
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    first,
                    UiMountedRgba8::new(220, 20, 60, 255),
                    UiMountedTextPaintSpanIdentity::from_runtime_mounting([17; 32]),
                ),
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    second,
                    UiMountedRgba8::new(30, 144, 255, 255),
                    UiMountedTextPaintSpanIdentity::from_runtime_mounting([29; 32]),
                ),
            ]),
            profile: worth_ui_host_contract::UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 1,
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
            capability_profile_digest: 11,
        },
    )
    .unwrap();
    let command = UiMountedPaintCommandIdentity::semantic_text(&mechanic);
    let damage = [worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting(bounds)];
    (layout, mechanic, command, damage)
}

fn range_contains(owner: UiTextOriginalRange, candidate: UiTextOriginalRange) -> bool {
    owner.start() <= candidate.start() && owner.end() >= candidate.end()
}
