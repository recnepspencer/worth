use std::sync::Arc;

use super::super::{order, Context};
use crate::*;

pub(super) fn surface_affinity(context: &Context) -> UiMountedPresentationAffinity {
    UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
        predecessor: context.predecessor,
        successor: context.frame,
        surface: context.surface,
        binding: context.binding,
        content: context.content,
        baseline: context.requirement.baseline(),
        production_cost: Default::default(),
    })
    .affinity()
}

pub(super) fn empty_successor(context: &Context) -> UiMountedAppearanceFrame {
    UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        [],
        order(context),
    )
    .unwrap()
}

pub(super) fn surface_at(
    frame: UiMountedFrameIdentity,
    instance: UiMountedInstanceIdentity,
) -> UiMountedSurfaceAppearanceMechanic {
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let bounds = UiAppearanceAllocationBounds::new(0, 0, 32, 32).unwrap();
    UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(instance),
            bounds,
            clip: UiAppearanceClip::new(0, 0, 32, 32).unwrap(),
            surface_paint_order: 0,
            radii: UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [UiAppearanceLogicalLength::ZERO; 4],
            ),
            border_edges: UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                0, 0, 0, 255,
            ])),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

pub(super) fn work_with_manifest(
    context: &Context,
    posture: UiMountedAppearanceWorkPosture,
    predecessor: impl IntoIterator<Item = UiMountedAppearanceMechanicIdentity>,
    mechanics: impl IntoIterator<Item = UiMountedAppearanceMechanic>,
    changes: impl IntoIterator<Item = UiMountedAppearanceMechanicChange>,
    damage: impl IntoIterator<Item = UiAppearanceDamageRegion>,
) -> UiMountedAppearanceWork {
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting(predecessor, []).unwrap();
    let successor = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        mechanics,
        order(context),
    )
    .unwrap();
    UiMountedAppearanceWork::from_runtime_mounting(
        posture,
        Some(context.predecessor),
        Some(manifest),
        successor,
        changes,
        damage,
        false,
    )
    .unwrap()
}

pub(super) fn removed_surface_fragment(context: &Context) -> UiUnpublishedAppearanceFragment {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let predecessor = surface_at(context.predecessor, instance);
    let receipt = predecessor.node_receipt();
    let identity = UiMountedAppearanceMechanic::Surface(predecessor).identity();
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting([identity.clone()], [])
            .unwrap();
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Delta,
        Some(context.predecessor),
        Some(manifest),
        empty_successor(context),
        [UiMountedAppearanceMechanicChange::Remove(identity)],
        [UiAppearanceDamageRegion::new(0, 0, 1, 1).unwrap()],
        false,
    )
    .unwrap();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(receipt),
            successor: None,
        },
        work,
        [],
        context.requirement,
        surface_affinity(context),
    )
    .unwrap()
}

pub(super) fn pointer_fragment(
    context: &Context,
    pointer: UiHostPointerIdentity,
) -> UiUnpublishedAppearanceFragment {
    let target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        pointer,
        context.surface,
        target,
        UiPointerAffordanceFamily::Activation,
    );
    let mounted = UiMountedAppearanceMechanic::Pointer(mechanic);
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting([mounted.identity()], [])
            .unwrap();
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        [mounted],
        order(context),
    )
    .unwrap();
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Unchanged,
        Some(context.predecessor),
        Some(manifest),
        frame,
        [],
        [],
        false,
    )
    .unwrap();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: context.surface,
            pointer,
        },
        work,
        [],
        context.requirement,
        surface_affinity(context),
    )
    .unwrap()
}

pub(super) fn mixed_text_row(
    context: &Context,
    instance: UiMountedInstanceIdentity,
    receipt: UiMountedNodeReceiptIdentity,
    adopted: UiMountedTextPaintSpanIdentity,
    retained: UiMountedTextPaintSpanIdentity,
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
            layout: mixed_layout(),
            slot: UiSemanticTextSlot::Value,
            collection_row: None,
            foregrounds: Arc::from([
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    UiTextOriginalRange::new(0, 3).unwrap(),
                    UiMountedRgba8::new(1, 2, 3, 255),
                    adopted,
                ),
                UiMountedTextForegroundSpan::from_runtime_mounting(
                    UiTextOriginalRange::new(3, 6).unwrap(),
                    UiMountedRgba8::new(4, 5, 6, 128),
                    retained,
                ),
            ]),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 7,
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
            capability_profile_digest: 11,
        },
    )
    .unwrap()
}

fn mixed_layout() -> UiQualifiedTextLayoutView<'static> {
    let source = "ONLINE";
    let style = |range| {
        UiQualifiedTextStyleRecord::from_text_mechanics(UiQualifiedTextStyleInput {
            original_range: range,
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
        })
    };
    let styles: &'static [UiQualifiedTextStyleRecord] = Box::leak(Box::new([
        style(UiTextOriginalRange::new(0, 3).unwrap()),
        style(UiTextOriginalRange::new(3, 6).unwrap()),
    ]));
    UiQualifiedTextLayoutView::from_text_mechanics(UiQualifiedTextLayoutViewInput {
        request_identity: UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([16; 32]),
        identity: UiQualifiedTextLayoutIdentity::from_text_mechanics([17; 32]),
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
