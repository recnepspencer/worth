use std::sync::Arc;

use super::*;
use crate::*;

struct Context {
    frame: UiMountedFrameIdentity,
    predecessor: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    content: UiMountedContentGeneration,
    attempt: UiMountedPresentationAttemptIdentity,
    requirement: UiMountedSurfaceBindingRequirement,
}

fn context() -> Context {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let predecessor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let content = UiMountedContentGeneration::mint_unbound().unwrap();
    let attempt = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let requirement = UiMountedSurfaceBindingRequirement::new(
        surface,
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        binding,
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::RecordOnly,
    );
    Context {
        frame,
        predecessor,
        surface,
        binding,
        content,
        attempt,
        requirement,
    }
}

fn order(context: &Context) -> UiMountedOverlayOrderMechanic {
    UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        context.surface,
        context.attempt,
        1,
        1,
        [],
    )
    .unwrap()
}

fn surface_work(
    context: &Context,
    mechanic: UiMountedSurfaceAppearanceMechanic,
) -> (UiMountedNodeReceiptIdentity, UiMountedAppearanceWork) {
    let receipt = mechanic.node_receipt();
    let identity = UiMountedAppearanceMechanic::Surface(mechanic.clone()).identity();
    let predecessor_manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting([identity], []).unwrap();
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        [UiMountedAppearanceMechanic::Surface(mechanic)],
        order(context),
    )
    .unwrap();
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Unchanged,
        Some(context.predecessor),
        Some(predecessor_manifest),
        frame,
        [],
        [],
        false,
    )
    .unwrap();
    (receipt, work)
}

fn surface_mechanic(
    context: &Context,
    instance: UiMountedInstanceIdentity,
) -> UiMountedSurfaceAppearanceMechanic {
    let issuer = UiMountedNodeReceiptIssuer::mint_for(context.frame).unwrap();
    let bounds = UiAppearanceAllocationBounds::new(0, 0, 32, 32).unwrap();
    UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(instance),
            bounds,
            clip: UiAppearanceClip::new(0, 0, 32, 32).unwrap(),
            layer: UiMountedLayerProjection::Layer(UiMountedLayerReference::new(0)),
            radii: UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [UiAppearanceLogicalLength::ZERO; 4],
            ),
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                0, 0, 0, 255,
            ])),
            opacity: UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

fn node_fragment(
    context: &Context,
    instance: UiMountedInstanceIdentity,
) -> UiUnpublishedAppearanceFragment {
    let mechanic = surface_mechanic(context, instance);
    let (receipt, work) = surface_work(context, mechanic);
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        work,
        [],
        context.requirement,
        node_affinity(context, receipt),
    )
    .unwrap()
}

fn node_affinity(
    context: &Context,
    receipt: UiMountedNodeReceiptIdentity,
) -> UiMountedPresentationAffinity {
    UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
        predecessor: context.predecessor,
        successor: context.frame,
        surface: context.surface,
        binding: context.binding,
        content: context.content,
        baseline: context.requirement.baseline(),
        production_cost: Default::default(),
    })
    .with_successor_receipt_affinity(Some(UiMountedNodeReceiptAffinity::from_receipt(receipt)))
    .affinity()
}

#[test]
fn multiple_node_fragments_may_share_one_surface_when_mechanics_are_disjoint() {
    let context = context();
    let first = node_fragment(&context, UiMountedInstanceIdentity::mint_unbound().unwrap());
    let second = node_fragment(&context, UiMountedInstanceIdentity::mint_unbound().unwrap());
    let projection = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
        context.frame,
        context.attempt,
        [first, second],
    )
    .unwrap();

    assert_eq!(projection.fragments().len(), 2);
    assert_eq!(
        projection.fragments()[0]
            .work()
            .successor()
            .semantic_surface(),
        context.surface
    );
    assert_eq!(
        projection.fragments()[1]
            .work()
            .successor()
            .semantic_surface(),
        context.surface
    );
}

#[test]
fn duplicate_fragment_identity_is_denied_before_a_second_attribution_exists() {
    let context = context();
    let fragment = node_fragment(&context, UiMountedInstanceIdentity::mint_unbound().unwrap());
    let identity = fragment.identity();
    assert_eq!(
        UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.attempt,
            [fragment.clone(), fragment],
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::DuplicateFragmentIdentity(identity))
    );
}

#[test]
fn text_candidate_must_join_the_exact_mounted_foreground_span() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(context.frame).unwrap();
    let receipt = issuer.receipt_for(instance);
    let span = UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]);
    let text = text_row(&context, instance, receipt, span);
    let foreground = UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer,
            node_receipt: receipt,
            paint_span: span,
            foreground: UiMountedAppearanceColor::from_straight_srgba([255; 4]),
            opacity: UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap();
    let (_, work) = text_work(&context, foreground, receipt);
    let wrong_text = text_row(
        &context,
        instance,
        receipt,
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([8; 32]),
    );
    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: None,
                successor: Some(receipt),
            },
            work,
            [wrong_text],
            context.requirement,
            node_affinity(&context, receipt),
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateSpanMismatch)
    );
    assert!(UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        text_work(
            &context,
            UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
                UiMountedTextForegroundAppearanceCompletionInput {
                    issuer,
                    node_receipt: receipt,
                    paint_span: span,
                    foreground: UiMountedAppearanceColor::from_straight_srgba([255; 4]),
                    opacity: UiMountedAppearanceOpacity::ONE,
                    projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                        issuer, 1, 1,
                    )
                    .unwrap(),
                },
            )
            .unwrap(),
            receipt,
        )
        .1,
        [text],
        context.requirement,
        node_affinity(&context, receipt),
    )
    .is_ok());
}

fn text_work(
    context: &Context,
    foreground: UiMountedTextForegroundAppearanceMechanic,
    receipt: UiMountedNodeReceiptIdentity,
) -> (UiMountedNodeReceiptIdentity, UiMountedAppearanceWork) {
    let mechanic = UiMountedAppearanceMechanic::TextForeground(foreground);
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting([mechanic.identity()], [])
            .unwrap();
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        [mechanic],
        order(context),
    )
    .unwrap();
    (
        receipt,
        UiMountedAppearanceWork::from_runtime_mounting(
            UiMountedAppearanceWorkPosture::Unchanged,
            Some(context.predecessor),
            Some(manifest),
            frame,
            [],
            [],
            false,
        )
        .unwrap(),
    )
}

fn text_row(
    context: &Context,
    instance: UiMountedInstanceIdentity,
    receipt: UiMountedNodeReceiptIdentity,
    span: UiMountedTextPaintSpanIdentity,
) -> UiMountedSemanticTextMechanic {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 32.0,
        y: 32.0,
        width: 160.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap();
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: context.content,
            frame: context.frame,
            surface: context.surface,
            binding: context.binding,
            mounted_instance: instance,
            node_receipt: receipt,
            allocation_basis: UiMountedAllocationBasis::new(
                1,
                2,
                3,
                UiMountedTransformProjection::Identity,
            ),
            bounds,
            clip_bounds: bounds,
            origin_x: 32.0,
            origin_y: 40.0,
            text: Arc::from("ONLINE"),
            layout: inert_layout(),
            slot: UiSemanticTextSlot::Value,
            collection_row: None,
            foregrounds: Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                UiTextOriginalRange::new(0, 6).unwrap(),
                UiMountedRgba8::new(255, 255, 255, 255),
                span,
            )]),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 7,
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
            capability_profile_digest: 11,
        },
    )
    .unwrap()
}

fn inert_layout() -> UiQualifiedTextLayoutView<'static> {
    let source = "ONLINE";
    let styles: &'static [UiQualifiedTextStyleRecord] =
        Box::leak(Box::new([UiQualifiedTextStyleRecord::from_text_mechanics(
            UiQualifiedTextStyleInput {
                original_range: UiTextOriginalRange::new(0, 6).unwrap(),
                language: "und".into(),
                font_size_millipoints: 14_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                family_stack: Box::new([]),
                weight: 400,
                width_milli_percent: 100_000,
                slant: UiFontSlant::Upright,
                features: Box::new([]),
                variations: Box::new([]),
            },
        )]));
    UiQualifiedTextLayoutView::from_text_mechanics(UiQualifiedTextLayoutViewInput {
        request_identity: UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([6; 32]),
        identity: UiQualifiedTextLayoutIdentity::from_text_mechanics([7; 32]),
        source,
        graphemes: &[],
        word_boundaries: &[],
        styles,
        logical_runs: &[],
        glyphs: &[],
        lines: &[],
        visual_runs: &[],
        positioned_glyphs: &[],
        logical_bounds: UiTextRect::from_text_mechanics(0, 0, 80_000, 18_000).unwrap(),
        ink_bounds: UiTextRect::from_text_mechanics(0, 2_000, 78_000, 16_000).unwrap(),
        carets: &[],
        coverage: &[],
        cost: UiQualifiedTextCostRecord::default(),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        text_scale: UiTextScaleGeneration::new(1).unwrap(),
        width_basis: UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
    })
}

#[path = "unpublished_edge_tests.rs"]
mod edge;
