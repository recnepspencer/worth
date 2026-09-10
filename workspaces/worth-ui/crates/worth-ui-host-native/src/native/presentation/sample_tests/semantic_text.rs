#[path = "semantic_text/order.rs"]
mod order;

use std::sync::Arc;

use super::{build_plan, sample_with_bounds, viewport_box, DrawListWorld};
use crate::native::{
    presentation::{
        raster::UiNativeRasterBasis, UiNativeRasterOperation, UiNativeRetainedDrawList,
    },
    text_atlas::{
        UiNativeTextAtlas, UiNativeTextAtlasDemand, UiNativeTextAtlasExternalOutcome,
        UiNativeTextAtlasUpload,
    },
};
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiFontSlant,
    UiGlyphRasterDemandIdentity, UiGlyphRasterFractionalOrigin, UiGlyphRasterKey,
    UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize, UiGlyphRasterSource,
    UiGlyphRunView, UiGlyphRunViewInput, UiGlyphVariationCoordinates, UiMountedAllocationBasis,
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedRgba8, UiMountedSemanticTextCompletionInput,
    UiMountedSemanticTextMechanic, UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity,
    UiMountedTransformProjection, UiPositionedTextGlyphRecord, UiQualifiedFontFaceIdentity,
    UiQualifiedTextCostRecord, UiQualifiedTextLayoutIdentity, UiQualifiedTextLayoutRequestIdentity,
    UiQualifiedTextLayoutView, UiQualifiedTextLayoutViewInput, UiQualifiedTextStyleInput,
    UiQualifiedTextStyleRecord, UiSemanticTextProfile, UiSemanticTextSlot, UiTextOriginalRange,
    UiTextProfileGeneration, UiTextRect, UiTextScaleGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

#[test]
fn production_sample_plan_transforms_semantic_text_glyphs_with_portal_opacity() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let (command, run, key) = semantic_text(&world, frame);
    let identity = command.identity();
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        std::slice::from_ref(&command),
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[run],
    )
    .unwrap();
    let source = viewport_box(10.0, 10.0, 40.0, 20.0);
    let sampled = viewport_box(22.0, 16.0, 40.0, 20.0);
    let sample = sample_with_bounds(&world, frame, identity, source, sampled, 32_768);
    let atlas = populated_atlas(key);
    let basis = UiNativeRasterBasis::new([96, 64], 1.0);
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (mut replay, mut undo) = retained.stage_sample(&sample).unwrap();
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .unwrap();

    assert_eq!(
        replay
            .physical_text_regions
            .iter()
            .map(|r| r.physical_bounds())
            .collect::<Vec<_>>(),
        [[22.0, 18.0, 4.0, 4.0], [34.0, 24.0, 4.0, 4.0]]
    );
    let plan = build_plan(
        UiNativeRasterBasis::new([96, 64], 1.0),
        &mut retained,
        replay,
        0,
        &atlas,
    )
    .unwrap();
    let glyph = plan
        .operations
        .iter()
        .find_map(|operation| match operation {
            UiNativeRasterOperation::Glyph(glyph) => Some(*glyph),
            UiNativeRasterOperation::Clear(_)
            | UiNativeRasterOperation::FilledRect { .. }
            | UiNativeRasterOperation::Surface(_) => None,
        })
        .expect("sample replay produces a native glyph operation");

    assert_eq!(glyph.run.mechanic(), identity);
    assert_eq!(glyph.opacity, sample.changes()[0].opacity().factor());
    assert_eq!(glyph.target, [34.0, 24.0, 4.0, 4.0]);
    assert_eq!(retained.command(identity), Some(&command));
    let collapsed = sample_with_bounds(
        &world,
        frame,
        identity,
        source,
        viewport_box(22.0, 16.0, 0.0, 20.0),
        32_768,
    );
    let (mut replay, mut undo) = retained.stage_sample(&collapsed).unwrap();
    retained
        .refresh_physical_sample(&collapsed, &mut undo, basis, &mut replay)
        .unwrap();
    assert_eq!(
        replay
            .physical_text_regions
            .iter()
            .map(|r| r.physical_bounds())
            .collect::<Vec<_>>(),
        [[34.0, 24.0, 4.0, 4.0]],
        "collapse clears the sampled predecessor, not its base image"
    );
    let collapsed_plan = build_plan(basis, &mut retained, replay, 0, &atlas).unwrap();
    assert!(collapsed_plan
        .operations
        .iter()
        .all(|op| !matches!(op, UiNativeRasterOperation::Glyph(_))));
    retained.rollback_sample(undo).unwrap();
    assert_eq!(
        retained
            .physical_replay_for_damage(basis, [34.0, 24.0, 1.0, 1.0], &mut Default::default())
            .unwrap()
            .as_ref(),
        [identity]
    );
    let (mut retry, mut undo) = retained.stage_sample(&collapsed).unwrap();
    retained
        .refresh_physical_sample(&collapsed, &mut undo, basis, &mut retry)
        .unwrap();
    let retry_plan = build_plan(basis, &mut retained, retry, 0, &atlas).unwrap();
    assert_eq!(retry_plan.operations.len(), collapsed_plan.operations.len());
    for (retry, original) in retry_plan
        .operations
        .iter()
        .zip(collapsed_plan.operations.iter())
    {
        let (UiNativeRasterOperation::Clear(retry), UiNativeRasterOperation::Clear(original)) =
            (retry, original)
        else {
            panic!("collapsed text replay contains only clears");
        };
        assert_eq!(retry, original);
    }
    retained.rollback_sample(undo).unwrap();
    let repaint = sample_with_bounds(&world, frame, identity, source, sampled, 16_384);
    let (mut replay, mut undo) = retained.stage_sample(&repaint).unwrap();
    retained
        .refresh_physical_sample(&repaint, &mut undo, basis, &mut replay)
        .unwrap();
    assert_eq!(
        replay
            .physical_text_regions
            .iter()
            .map(|r| r.physical_bounds())
            .collect::<Vec<_>>(),
        [[34.0, 24.0, 4.0, 4.0]],
        "equal geometry still clears once for changed opacity"
    );
    retained.rollback_sample(undo).unwrap();
}

pub(in crate::native::presentation) fn semantic_text(
    world: &DrawListWorld,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
) -> (UiMountedPaintCommand, UiGlyphRunView, UiGlyphRasterKey) {
    let text: Arc<str> = Arc::from("A");
    let layout = inert_layout();
    let bounds = viewport_box(10.0, 10.0, 40.0, 20.0);
    let mechanic = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: world.content,
            frame,
            surface: world.surface,
            binding: world.binding,
            mounted_instance: world.first,
            node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame)
                .unwrap()
                .receipt_for(world.first),
            allocation_basis: UiMountedAllocationBasis::new(
                1,
                2,
                3,
                UiMountedTransformProjection::Identity,
            ),
            bounds,
            clip_bounds: bounds,
            origin_x: 22.0,
            origin_y: 18.0,
            text,
            layout,
            slot: UiSemanticTextSlot::Value,
            collection_row: None,
            foregrounds: Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                UiTextOriginalRange::new(0, 1).unwrap(),
                UiMountedRgba8::new(235, 238, 245, 255),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([9; 32]),
            )]),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 8,
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
            capability_profile_digest: 11,
        },
    )
    .unwrap();
    let identity = UiMountedPaintCommandIdentity::semantic_text(&mechanic);
    let key = raster_key();
    let run = UiGlyphRunView::from_text_mechanics(UiGlyphRunViewInput {
        mechanic: identity,
        layout: layout.identity(),
        paint_span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([9; 32]),
        original_range: UiTextOriginalRange::new(0, 1).unwrap(),
        foreground: UiMountedRgba8::new(235, 238, 245, 255),
        raster_key: key,
        origin_x_millipoints: 22_000,
        origin_y_millipoints: 18_000,
        line_index: 0,
        visual_run_index: 0,
        clip_bounds: bounds,
        layer_semantic_order: 8,
    });
    (
        UiMountedPaintCommand::SemanticText { identity, mechanic },
        run,
        key,
    )
}

fn inert_layout() -> UiQualifiedTextLayoutView<'static> {
    static GLYPHS: [UiPositionedTextGlyphRecord; 0] = [];
    let styles = Box::leak(Box::new([UiQualifiedTextStyleRecord::from_text_mechanics(
        UiQualifiedTextStyleInput {
            original_range: UiTextOriginalRange::new(0, 1).unwrap(),
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
        request_identity: UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([5; 32]),
        identity: UiQualifiedTextLayoutIdentity::from_text_mechanics([6; 32]),
        source: "A",
        graphemes: &[],
        word_boundaries: &[],
        styles,
        logical_runs: &[],
        glyphs: &[],
        lines: &[],
        visual_runs: &[],
        positioned_glyphs: &GLYPHS,
        logical_bounds: UiTextRect::from_text_mechanics(0, 0, 14_000, 18_000).unwrap(),
        ink_bounds: UiTextRect::from_text_mechanics(0, 0, 4_000, 4_000).unwrap(),
        carets: &[],
        coverage: &[],
        cost: UiQualifiedTextCostRecord::default(),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        text_scale: UiTextScaleGeneration::new(1).unwrap(),
        width_basis: worth_ui_host_contract::UiQualifiedTextLayoutWidthBasis::new(40_000).unwrap(),
    })
}

fn populated_atlas(key: UiGlyphRasterKey) -> UiNativeTextAtlas {
    let atlas = UiNativeTextAtlas::new();
    let demand = UiNativeTextAtlasDemand::from_native_geometry(
        UiGlyphRasterDemandIdentity::from_text_mechanics([7; 32]),
        key,
        4,
        4,
        16,
    );
    let plan = atlas.plan_demands(&[demand], &Default::default()).unwrap();
    let upload = UiNativeTextAtlasUpload::from_text_mechanics(key, 4, 4, 4, vec![255; 16], [8; 32]);
    let outcome = atlas.settle(plan, &[upload], UiNativeTextAtlasExternalOutcome::Submitted);
    assert!(matches!(
        outcome,
        crate::native::text_atlas::UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    atlas
}

fn raster_key() -> UiGlyphRasterKey {
    UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([2; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([3; 32], 0),
        glyph_id: 41,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(14_000).unwrap(),
        source: UiGlyphRasterSource::AlphaOutline,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}
