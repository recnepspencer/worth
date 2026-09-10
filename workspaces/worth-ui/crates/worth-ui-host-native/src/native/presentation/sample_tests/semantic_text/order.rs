//! Order-only replay consumes sampled images without retiring the Motion override.
use super::*;
use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedPaintOrderEdit, UiMountedPresentationDelta,
    UiMountedPresentationDeltaInput,
};

#[test]
fn order_only_text_damage_preserves_sampled_coverage_and_opacity_on_retry() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let (text, run, key) = semantic_text(&world, frame);
    let id = text.identity();
    let rect = world.rect(
        frame,
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        20.0,
        UiMountedRgba8::new(30, 60, 90, 255),
    );
    let rect = UiMountedPaintCommand::FilledRect {
        identity: UiMountedPaintCommandIdentity::filled_rect(&rect),
        mechanic: rect,
    };
    let order = [
        UiMountedPaintOrderIdentity::for_command(id),
        UiMountedPaintOrderIdentity::for_command(rect.identity()),
    ];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &[text, rect],
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[run],
    )
    .unwrap();
    let atlas = populated_atlas(key);
    let basis = UiNativeRasterBasis::new([96, 64], 1.0);
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let sample = sample_with_bounds(
        &world,
        frame,
        id,
        viewport_box(10.0, 10.0, 40.0, 20.0),
        viewport_box(22.0, 16.0, 40.0, 20.0),
        32_768,
    );
    let (mut replay, mut undo) = retained.stage_sample(&sample).unwrap();
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .unwrap();
    let change = sample.changes()[0];
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: frame,
        successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![],
        nodes: vec![],
        order: vec![UiMountedPaintOrderEdit::place_after(
            order[0],
            Some(order[1]),
        )],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[order[1], order[0]]),
        damage: vec![],
        auxiliary: None,
        production_cost: Default::default(),
    });
    for _attempt in 0..2 {
        let (mut replay, mut undo) = retained.stage_delta(&delta, &[]).unwrap();
        retained
            .refresh_physical_delta(&delta, &mut undo, basis, &atlas, &mut replay)
            .unwrap();
        assert_eq!(
            replay
                .physical_text_regions
                .iter()
                .map(|r| r.physical_bounds())
                .collect::<Vec<_>>(),
            [[34.0, 24.0, 4.0, 4.0]],
            "order consumes sampled current, never base"
        );
        assert_eq!(retained.sample_override(id), Some(change));
        let plan = build_plan(basis, &mut retained, replay, 0, &atlas).unwrap();
        let glyphs = plan
            .operations
            .iter()
            .filter_map(|op| match op {
                UiNativeRasterOperation::Glyph(glyph) => Some(glyph),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].target, [34.0, 24.0, 4.0, 4.0]);
        assert_eq!(glyphs[0].opacity, change.opacity().factor());
        retained.rollback_delta(undo).unwrap();
        assert_eq!(retained.sample_override(id), Some(change));
        assert_eq!(
            retained
                .physical_replay_for_damage(basis, [34.0, 24.0, 1.0, 1.0], &mut Default::default())
                .unwrap()
                .last(),
            Some(&id)
        );
    }
}
