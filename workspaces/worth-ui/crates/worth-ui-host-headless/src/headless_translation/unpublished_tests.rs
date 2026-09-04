use std::sync::Arc;

use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiAppearanceLogicalLength,
    UiFontCollectionGeneration, UiFontSlant, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
    UiMountedAppearanceColor, UiMountedAppearanceFrame, UiMountedAppearanceMechanic,
    UiMountedAppearancePredecessorManifest, UiMountedAppearanceWork,
    UiMountedAppearanceWorkPosture, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedLayerProjection, UiMountedLayerReference, UiMountedNodeAppearanceAttribution,
    UiMountedNodeReceiptIssuer, UiMountedPresentationAttemptIdentity,
    UiMountedPresentationUnchanged, UiMountedPresentationUnchangedInput, UiMountedRgba8,
    UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic,
    UiMountedSurfaceAppearanceCompletionInput, UiMountedSurfaceAppearanceMechanic,
    UiMountedSurfaceBindingRequirement, UiMountedSurfacePaint,
    UiMountedTextForegroundAppearanceCompletionInput, UiMountedTextForegroundAppearanceMechanic,
    UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity, UiMountedTransformProjection,
    UiQualifiedTextCostRecord, UiQualifiedTextLayoutIdentity, UiQualifiedTextLayoutRequestIdentity,
    UiQualifiedTextLayoutView, UiQualifiedTextLayoutViewInput, UiQualifiedTextLayoutWidthBasis,
    UiQualifiedTextStyleInput, UiQualifiedTextStyleRecord, UiSemanticSurfaceIdentity,
    UiSemanticTextProfile, UiSemanticTextSlot, UiTextOriginalRange, UiTextProfileGeneration,
    UiTextRect, UiTextScaleGeneration, UiUnpublishedAppearanceFragment,
    UiUnpublishedAppearanceFragmentIdentity, UiUnpublishedAppearanceFrameProjection,
    WorthUiHostCapabilityObservationGeneration,
};

#[path = "unpublished_text_support.rs"]
mod text_support;

struct Context {
    frame: UiMountedFrameIdentity,
    predecessor: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    content: worth_ui_host_contract::UiMountedContentGeneration,
    presentation: UiMountedPresentationAttemptIdentity,
    requirement: UiMountedSurfaceBindingRequirement,
}

fn context() -> Context {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let predecessor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let content = worth_ui_host_contract::UiMountedContentGeneration::mint_unbound().unwrap();
    let presentation = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
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
        presentation,
        requirement,
    }
}

fn order(context: &Context) -> worth_ui_host_contract::UiMountedOverlayOrderMechanic {
    worth_ui_host_contract::UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        context.surface,
        context.presentation,
        1,
        1,
        [],
    )
    .unwrap()
}

fn surface(
    context: &Context,
    instance: UiMountedInstanceIdentity,
) -> (
    UiMountedSurfaceAppearanceMechanic,
    UiMountedNodeReceiptIssuer,
) {
    let issuer = UiMountedNodeReceiptIssuer::mint_for(context.frame).unwrap();
    let bounds = UiAppearanceAllocationBounds::new(0, 0, 32, 32).unwrap();
    let surface = UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(instance),
            bounds,
            clip: UiAppearanceClip::new(0, 0, 32, 32).unwrap(),
            layer: UiMountedLayerProjection::Layer(UiMountedLayerReference::new(0)),
            radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [UiAppearanceLogicalLength::ZERO; 4],
            ),
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                0, 0, 0, 255,
            ])),
            opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap();
    (surface, issuer)
}

fn fragment(
    context: &Context,
    instance: UiMountedInstanceIdentity,
) -> UiUnpublishedAppearanceFragment {
    let (surface, issuer) = surface(context, instance);
    let receipt = issuer.receipt_for(instance);
    let identity = UiMountedAppearanceMechanic::Surface(surface.clone()).identity();
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting([identity], []).unwrap();
    let successor = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        [UiMountedAppearanceMechanic::Surface(surface)],
        order(context),
    )
    .unwrap();
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Unchanged,
        Some(context.predecessor),
        Some(manifest),
        successor,
        [],
        [],
        false,
    )
    .unwrap();
    let affinity =
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor: context.predecessor,
            successor: context.frame,
            surface: context.surface,
            binding: context.binding,
            content: context.content,
            baseline: context.requirement.baseline(),
            production_cost: Default::default(),
        })
        .with_successor_receipt_affinity(Some(issuer.receipt_affinity()))
        .affinity();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        work,
        [],
        context.requirement,
        affinity,
    )
    .unwrap()
}

fn fragment_with_text_candidates(
    context: &Context,
    instance: UiMountedInstanceIdentity,
) -> UiUnpublishedAppearanceFragment {
    let (surface, issuer) = surface(context, instance);
    let receipt = issuer.receipt_for(instance);
    let spans = [
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([101; 32]),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([102; 32]),
    ];
    let foregrounds = spans.map(|span| {
        UiMountedAppearanceMechanic::TextForeground(text_foreground(issuer, receipt, span))
    });
    let candidates = [
        text_candidate(
            context,
            instance,
            receipt,
            spans[0],
            UiSemanticTextSlot::Value,
            UiTextOriginalRange::new(0, 3).unwrap(),
        ),
        text_candidate(
            context,
            instance,
            receipt,
            spans[1],
            UiSemanticTextSlot::Posture,
            UiTextOriginalRange::new(3, 6).unwrap(),
        ),
    ];
    let mut manifest_identities =
        vec![UiMountedAppearanceMechanic::Surface(surface.clone()).identity()];
    manifest_identities.extend(
        foregrounds
            .iter()
            .map(UiMountedAppearanceMechanic::identity),
    );
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting(manifest_identities, [])
            .unwrap();
    let mut successor_mechanics = vec![UiMountedAppearanceMechanic::Surface(surface)];
    successor_mechanics.extend(foregrounds);
    let successor = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        successor_mechanics,
        order(context),
    )
    .unwrap();
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Unchanged,
        Some(context.predecessor),
        Some(manifest),
        successor,
        [],
        [],
        false,
    )
    .unwrap();
    let predecessor_issuer = UiMountedNodeReceiptIssuer::mint_for(context.predecessor).unwrap();
    let affinity =
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor: context.predecessor,
            successor: context.frame,
            surface: context.surface,
            binding: context.binding,
            content: context.content,
            baseline: context.requirement.baseline(),
            production_cost: Default::default(),
        })
        .with_successor_receipt_affinity(Some(issuer.receipt_affinity()))
        .affinity();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(predecessor_issuer.receipt_for(instance)),
            successor: Some(receipt),
        },
        work,
        candidates,
        context.requirement,
        affinity,
    )
    .unwrap()
}

fn text_foreground(
    issuer: UiMountedNodeReceiptIssuer,
    receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    span: UiMountedTextPaintSpanIdentity,
) -> UiMountedTextForegroundAppearanceMechanic {
    UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer,
            node_receipt: receipt,
            paint_span: span,
            foreground: UiMountedAppearanceColor::from_straight_srgba([9, 8, 7, 255]),
            opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 2, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

fn text_candidate(
    context: &Context,
    instance: UiMountedInstanceIdentity,
    receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    span: UiMountedTextPaintSpanIdentity,
    slot: UiSemanticTextSlot,
    range: UiTextOriginalRange,
) -> UiMountedSemanticTextMechanic {
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 32.0,
            y: 32.0,
            width: 160.0,
            height: 96.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .unwrap();
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: context.content,
            frame: context.frame,
            surface: context.surface,
            binding: context.binding,
            mounted_instance: instance,
            node_receipt: receipt,
            allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis::new(
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
            layout: text_support::qualified_layout(range),
            slot,
            collection_row: None,
            foregrounds: Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                range,
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

#[test]
fn facade_translates_actual_host_projection_in_fragment_order() {
    let context = context();
    let first =
        fragment_with_text_candidates(&context, UiMountedInstanceIdentity::mint_unbound().unwrap());
    let expected_candidates = first.text_candidates().to_vec();
    let second = fragment(&context, UiMountedInstanceIdentity::mint_unbound().unwrap());
    let first_identity = first.identity();
    let second_identity = second.identity();
    let projection = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
        context.frame,
        context.presentation,
        [first, second],
    )
    .unwrap();

    let transcript =
        crate::translate_unpublished_appearance_for_certification(&projection).unwrap();

    assert_eq!(transcript.frame(), context.frame);
    assert_eq!(transcript.presentation(), context.presentation);
    assert_eq!(transcript.fragments().len(), 2);
    assert_eq!(transcript.fragments()[0].identity(), first_identity);
    assert_eq!(transcript.fragments()[1].identity(), second_identity);
    assert_eq!(
        transcript.fragments()[0].text_candidates(),
        expected_candidates.as_slice()
    );
    assert_eq!(
        transcript.fragments()[0]
            .text_candidates()
            .iter()
            .map(|candidate| (
                candidate.node_receipt(),
                candidate.foregrounds()[0].identity()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                expected_candidates[0].node_receipt(),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([101; 32])
            ),
            (
                expected_candidates[1].node_receipt(),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([102; 32])
            ),
        ]
    );
    for (index, fragment) in transcript.fragments().iter().enumerate() {
        assert_eq!(
            fragment.work().posture(),
            UiMountedAppearanceWorkPosture::Unchanged
        );
        assert_eq!(fragment.work().predecessor(), Some(context.predecessor));
        assert!(fragment.work().changes().is_empty());
        assert!(fragment.work().damage().is_empty());
        assert_eq!(fragment.surface_binding(), context.requirement);
        assert_eq!(
            fragment.work().successor().mechanics().len(),
            if index == 0 {
                expected_candidates.len() + 1
            } else {
                1
            }
        );
        assert_eq!(
            fragment.work().successor().overlay_order(),
            &order(&context)
        );
    }
    assert!(transcript.fragments()[1].text_candidates().is_empty());
}
