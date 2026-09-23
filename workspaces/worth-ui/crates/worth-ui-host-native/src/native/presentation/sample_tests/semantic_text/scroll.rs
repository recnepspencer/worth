use super::*;
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedLogicalDamage, UiMountedPresentationOpacity,
    UiMountedPresentationSample, UiMountedPresentationSampleChange,
    UiMountedPresentationSampleInput, UiMountedPresentationTransform,
};

#[test]
fn scroll_sample_reveals_retained_glyph_pixels_through_a_stationary_clip_and_rolls_back() {
    assert_reveal(
        10.0,
        -4.0,
        Some([22.0, 18.0, 4.0, 1.0]),
        [22.0, 14.0, 4.0, 4.0],
    );
}

#[test]
fn scroll_sample_reveals_a_fully_offscreen_retained_row_and_rolls_back() {
    assert_reveal(40.0, 25.0, None, [22.0, 43.0, 4.0, 4.0]);
}

fn assert_reveal(viewport_y: f32, delta: f32, initial: Option<[f32; 4]>, revealed: [f32; 4]) {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let (command, original, key) = semantic_text(&world, frame);
    let UiMountedPaintCommand::SemanticText { identity, mechanic } = command else {
        unreachable!()
    };
    // The A image covers y=18..22. The viewport can hide part or all of it.
    let mechanic = mechanic
        .clipped_to_appearance_ancestor(
            UiAppearanceClip::new(10_000, (viewport_y * 1_000.0) as i32, 40_000, 9_000).unwrap(),
        )
        .unwrap()
        .unwrap();
    let clip = mechanic.clip_bounds();
    let run = UiGlyphRunView::from_text_mechanics(UiGlyphRunViewInput {
        mechanic: identity,
        layout: original.layout_identity(),
        paint_span: original.paint_span(),
        original_range: original.original_range(),
        foreground: original.foreground(),
        raster_key: key,
        origin_x_millipoints: original.origin_x_millipoints(),
        origin_y_millipoints: original.origin_y_millipoints(),
        line_index: original.line_index(),
        visual_run_index: original.visual_run_index(),
        clip_bounds: clip,
        intrinsic_clip_bounds: mechanic.intrinsic_clip_bounds(),
        layer_semantic_order: original.layer_semantic_order(),
    });
    let command = UiMountedPaintCommand::SemanticText { identity, mechanic };
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &[command],
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[run],
    )
    .unwrap();
    let basis = UiNativeRasterBasis::new([96, 64], 1.0);
    let atlas = populated_atlas(key);
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let before = retained
        .plan_text_commands(identity, &atlas, basis)
        .unwrap();
    assert_eq!(before.first().map(|glyph| glyph.target), initial);
    let source = viewport_box(10.0, 10.0, 40.0, 20.0);
    let target = viewport_box(10.0, 10.0 + delta, 40.0, 20.0);
    let viewport = viewport_box(10.0, viewport_y, 40.0, 9.0);
    let sample =
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            production_cost: Default::default(),
            changes: vec![
                UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
                    identity,
                    UiMountedPresentationTransform::from_runtime_sampling(source, target).unwrap(),
                    UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                    viewport,
                )
                .unwrap(),
            ],
            damage: vec![UiMountedLogicalDamage::from_runtime_mounting(viewport)],
        })
        .unwrap();
    let (plan, undo) = crate::native::presentation::sample::prepare_sample_plan(
        basis,
        &sample,
        &atlas,
        &mut retained,
    )
    .unwrap_or_else(|_| panic!("Scroll sample crosses native physical preparation"));
    assert!(
        plan.operations.iter().any(|operation| matches!(operation,
            UiNativeRasterOperation::Glyph(glyph) if glyph.target == revealed
        )),
        "scrolling reveals the full A, not the single row clipped before movement"
    );
    if let Some(before) = before.first() {
        assert_eq!(
            retained
                .plan_text_commands(identity, &atlas, basis)
                .unwrap()[0]
                .texture_uv[3],
            before.texture_uv[3] * 4.0
        );
    }
    retained.rollback_sample(undo).unwrap();
    assert_eq!(
        retained
            .plan_text_commands(identity, &atlas, basis)
            .unwrap(),
        before
    );
    assert!(retained.sample_override(identity).is_none());
}
