use std::sync::Arc;

use super::{
    UiMountedCollectionRowCorrelation, UiMountedSemanticTextCompletionDenial,
    UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic,
    UiMountedSemanticTextTable, UiMountedSemanticTextTableDenial, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiSemanticTextProfile, UiSemanticTextSlot,
};
use crate::{
    UiFontCollectionGeneration, UiFontSlant, UiMountedAllocationBasis, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedContentGeneration, UiMountedCoordinateSpace,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer, UiMountedRgba8,
    UiMountedTransformProjection, UiQualifiedTextCostRecord, UiQualifiedTextLayoutIdentity,
    UiQualifiedTextLayoutView, UiQualifiedTextLayoutViewInput, UiQualifiedTextStyleInput,
    UiQualifiedTextStyleRecord, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    UiTextProfileGeneration, UiTextRect, UiTextScaleGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

#[path = "semantic_text/frame_affinity_tests.rs"]
mod frame_affinity_tests;

#[test]
fn completion_preserves_runtime_owned_semantic_text_meaning() {
    let input = fixture();
    let row = complete(input.clone());

    assert_eq!(row.content_generation(), input.content_generation);
    assert_eq!(row.frame(), input.frame);
    assert_eq!(row.surface(), input.surface);
    assert_eq!(row.binding(), input.binding);
    assert_eq!(row.mounted_instance(), input.mounted_instance);
    assert_eq!(row.node_receipt(), input.node_receipt);
    assert_eq!(row.allocation_basis(), input.allocation_basis);
    assert_eq!(row.bounds(), input.bounds);
    assert_eq!(row.clip_bounds(), input.clip_bounds);
    assert_eq!((row.origin_x(), row.origin_y()), (32.0, 40.0));
    assert_eq!(row.text(), "ONLINE");
    assert_eq!(row.slot(), UiSemanticTextSlot::Value);
    assert_eq!(row.profile(), UiSemanticTextProfile::BodyDefault);
    assert_eq!(row.capability_generation(), input.capability_generation);
    assert_eq!(
        row.capability_profile_digest(),
        input.capability_profile_digest
    );
}

#[test]
fn geometry_origin_and_receipt_mismatches_are_typed_denials() {
    let mut input = fixture();
    input.bounds = canonical_box(32.0, 32.0, 0.0, 96.0);
    input.clip_bounds = input.bounds;
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::NonAreaGeometry,
    );

    let mut input = fixture();
    input.clip_bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 32.0,
        y: 32.0,
        width: 159.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::Window,
    })
    .unwrap();
    assert_denial(input, UiMountedSemanticTextCompletionDenial::ClipMismatch);

    for (origin_x, origin_y) in [(f32::NAN, 40.0), (31.0, 40.0), (32.0, 129.0)] {
        let mut input = fixture();
        input.origin_x = origin_x;
        input.origin_y = origin_y;
        assert_denial(
            input,
            UiMountedSemanticTextCompletionDenial::InvalidTextOrigin,
        );
    }

    let mut input = fixture();
    let foreign_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    input.node_receipt = UiMountedNodeReceiptIssuer::mint_for(foreign_frame)
        .unwrap()
        .receipt_for(input.mounted_instance);
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::NodeReceiptFrameMismatch,
    );

    let mut input = fixture();
    input.node_receipt = UiMountedNodeReceiptIssuer::mint_for(input.frame)
        .unwrap()
        .receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap());
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::NodeReceiptInstanceMismatch,
    );
}

#[test]
fn digest_changes_with_content_context_and_placement() {
    let baseline = fixture();
    let digest = complete(baseline.clone()).semantic_digest();
    let mut variants = Vec::new();
    variants.push(with(&baseline, |input| {
        set_text(input, Arc::from("UPDATED"));
    }));
    variants.push(with(&baseline, |input| input.origin_y = 41.0));
    variants.push(with(&baseline, |input| {
        input.slot = UiSemanticTextSlot::Posture
    }));
    variants.push(with(&baseline, |input| input.layer_semantic_order = 8));
    variants.push(with(&baseline, |input| {
        input.capability_generation = WorthUiHostCapabilityObservationGeneration::new(8)
    }));
    variants.push(with(&baseline, |input| {
        input.capability_profile_digest = 10
    }));
    variants.push(with(&baseline, |input| {
        input.content_generation = UiMountedContentGeneration::mint_unbound().unwrap()
    }));

    for variant in variants {
        assert_ne!(complete(variant).semantic_digest(), digest);
    }
}

#[test]
fn retained_text_equivalence_excludes_lineage_but_rejects_changed_paint_inputs() {
    let baseline = fixture();
    let row = complete(baseline.clone());
    let advanced = with(&baseline, |input| {
        input.content_generation = UiMountedContentGeneration::mint_unbound().unwrap();
        input.frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        input.node_receipt = UiMountedNodeReceiptIssuer::mint_for(input.frame)
            .unwrap()
            .receipt_for(input.mounted_instance);
    });
    assert!(row.same_retained_paint_meaning(&complete(advanced)));

    let variants = [
        with(&baseline, |input| input.origin_y = 41.0),
        with(&baseline, |input| {
            input.bounds = canonical_box(31.0, 32.0, 160.0, 96.0)
        }),
        with(&baseline, |input| {
            input.layout = inert_layout_with_identity("ONLINE", 8)
        }),
        with(&baseline, |input| input.layer_semantic_order = 8),
        with(&baseline, |input| {
            input.capability_generation = WorthUiHostCapabilityObservationGeneration::new(8)
        }),
        with(&baseline, |input| input.capability_profile_digest = 10),
        with(&baseline, |input| {
            input.foregrounds = Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                crate::UiTextOriginalRange::from_text_mechanics(0, 6).unwrap(),
                UiMountedRgba8::new(254, 255, 255, 255),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
            )])
        }),
    ];
    for variant in variants {
        assert!(!row.same_retained_paint_meaning(&complete(variant)));
    }
}

#[test]
fn collection_slot_and_row_correlation_are_atomic() {
    let mut missing_identity = fixture();
    missing_identity.slot = UiSemanticTextSlot::CollectionValue {
        selected_field_ordinal: 0,
    };
    assert_denial(
        missing_identity,
        UiMountedSemanticTextCompletionDenial::CollectionIdentityMismatch,
    );

    let mut scalar_with_identity = fixture();
    scalar_with_identity.collection_row = Some(
        UiMountedCollectionRowCorrelation::from_runtime_mounting([0xA1; 32]),
    );
    assert_denial(
        scalar_with_identity,
        UiMountedSemanticTextCompletionDenial::CollectionIdentityMismatch,
    );

    let mut collection = fixture();
    collection.slot = UiSemanticTextSlot::CollectionValue {
        selected_field_ordinal: 0,
    };
    collection.collection_row = Some(UiMountedCollectionRowCorrelation::from_runtime_mounting(
        [0xA1; 32],
    ));
    let row = complete(collection);
    assert_eq!(
        row.collection_row().unwrap().correlation_digest(),
        [0xA1; 32]
    );
}

#[test]
fn row_and_table_byte_caps_are_enforced() {
    assert_eq!(UiMountedSemanticTextMechanic::MAX_CONTENT_BYTES, 65_536);
    assert_eq!(UiMountedSemanticTextTable::MAX_ROWS, 8_192);
    assert_eq!(UiMountedSemanticTextTable::MAX_BYTES, 8 * 1_024 * 1_024);

    let mut input = fixture();
    let oversized: Arc<str> =
        Arc::from("x".repeat(UiMountedSemanticTextMechanic::MAX_CONTENT_BYTES + 1));
    set_text(&mut input, Arc::clone(&oversized));
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::ContentCapacityExceeded,
    );

    let mut row_input = fixture();
    let maximum: Arc<str> = Arc::from("x".repeat(UiMountedSemanticTextMechanic::MAX_CONTENT_BYTES));
    set_text(&mut row_input, Arc::clone(&maximum));
    let row = complete(row_input);
    let rows = vec![
        row;
        UiMountedSemanticTextTable::MAX_BYTES
            / UiMountedSemanticTextMechanic::MAX_CONTENT_BYTES
            + 1
    ];
    assert_eq!(
        UiMountedSemanticTextTable::from_runtime_mounting(rows),
        Err(UiMountedSemanticTextTableDenial::ByteCapacityExceeded)
    );

    let row = complete(fixture());
    let rows = vec![row; UiMountedSemanticTextTable::MAX_ROWS + 1];
    assert_eq!(
        UiMountedSemanticTextTable::from_runtime_mounting(rows),
        Err(UiMountedSemanticTextTableDenial::CapacityExceeded)
    );
}

pub(in crate::mounted_projection) fn fixture() -> UiMountedSemanticTextCompletionInput<'static> {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let bounds = canonical_box(32.0, 32.0, 160.0, 96.0);
    UiMountedSemanticTextCompletionInput {
        content_generation: UiMountedContentGeneration::mint_unbound().unwrap(),
        frame,
        surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        mounted_instance,
        node_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
            .unwrap()
            .receipt_for(mounted_instance),
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
        layout: inert_layout("ONLINE"),
        slot: UiSemanticTextSlot::Value,
        collection_row: None,
        foregrounds: Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
            crate::UiTextOriginalRange::from_text_mechanics(0, 6).unwrap(),
            UiMountedRgba8::new(255, 255, 255, 255),
            UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
        )]),
        profile: UiSemanticTextProfile::BodyDefault,
        layer_semantic_order: 7,
        capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
        capability_profile_digest: 9,
    }
}

#[test]
fn raw_text_cannot_impersonate_a_different_qualified_layout() {
    let mut input = fixture();
    input.text = Arc::from("UPDATED");
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::QualifiedLayoutSourceMismatch,
    );
}

#[test]
fn foreground_spans_must_match_the_canonical_layout_itemization() {
    let mut input = fixture();
    input.foregrounds = Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
        crate::UiTextOriginalRange::new(0, 5).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
    )]);
    assert_denial(
        input,
        UiMountedSemanticTextCompletionDenial::ForegroundSpanMismatch,
    );
}

fn inert_layout(source: &str) -> UiQualifiedTextLayoutView<'static> {
    inert_layout_with_identity(source, 7)
}

fn inert_layout_with_identity(source: &str, identity: u8) -> UiQualifiedTextLayoutView<'static> {
    let source: &'static str = Box::leak(source.to_owned().into_boxed_str());
    let styles: &'static [UiQualifiedTextStyleRecord] = if source.is_empty() {
        &[]
    } else {
        Box::leak(Box::new([UiQualifiedTextStyleRecord::from_text_mechanics(
            UiQualifiedTextStyleInput {
                original_range: crate::UiTextOriginalRange::new(0, source.len() as u32).unwrap(),
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
        )]))
    };
    UiQualifiedTextLayoutView::from_text_mechanics(UiQualifiedTextLayoutViewInput {
        request_identity: crate::UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([6; 32]),
        identity: UiQualifiedTextLayoutIdentity::from_text_mechanics([identity; 32]),
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
        width_basis: crate::UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
    })
}

fn complete(input: UiMountedSemanticTextCompletionInput<'_>) -> UiMountedSemanticTextMechanic {
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting(input).unwrap()
}

fn assert_denial(
    input: UiMountedSemanticTextCompletionInput<'_>,
    expected: UiMountedSemanticTextCompletionDenial,
) {
    assert_eq!(
        UiMountedSemanticTextMechanic::complete_from_runtime_mounting(input),
        Err(expected)
    );
}

fn with<'layout>(
    input: &UiMountedSemanticTextCompletionInput<'layout>,
    mutate: impl FnOnce(&mut UiMountedSemanticTextCompletionInput<'layout>),
) -> UiMountedSemanticTextCompletionInput<'layout> {
    let mut input = input.clone();
    mutate(&mut input);
    input
}

fn set_text(input: &mut UiMountedSemanticTextCompletionInput<'static>, text: Arc<str>) {
    input.layout = inert_layout(&text);
    input.foregrounds = Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
        crate::UiTextOriginalRange::new(0, text.len() as u32).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
    )]);
    input.text = text;
}

fn canonical_box(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}
