use super::*;

#[test]
fn no_paint_sample_preserves_retained_commands_and_requires_exact_affinity() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(
        frame,
        world.first,
        10.0,
        UiMountedRgba8::new(30, 60, 90, 255),
    );
    let initial = world.initial(frame, [rect]);
    let identity = command(rect).identity();
    let semantic = command(rect);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let basis = UiNativeRasterBasis::new([100, 100], 1.0);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let prior = sample(&world, frame, identity, 10.0, 30.0, 32_768);
    super::super::prepare_sample_plan(basis, &prior, &atlas, &mut retained)
        .unwrap_or_else(|_| panic!("the first sample reaches retained paint"));
    let previous = retained.sample_override(identity);
    let empty = |frame| {
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            production_cost: Default::default(),
            changes: Vec::new(),
            damage: Vec::new(),
        })
        .unwrap()
    };
    let (plan, undo) =
        super::super::prepare_sample_plan(basis, &empty(frame), &atlas, &mut retained)
            .unwrap_or_else(|_| panic!("exact same-frame no-paint sample is admitted"));
    assert!(plan.operations.is_empty());
    assert!(!plan.clear_retained_target);
    assert_eq!(plan.cost.presented_pixels(), 0);
    assert_eq!(plan.cost.gpu_writes(), 0);
    assert_eq!(retained.command(identity), Some(&semantic));
    assert_eq!(retained.sample_override(identity), previous);
    retained.rollback_sample(undo).unwrap();
    assert_eq!(retained.sample_override(identity), previous);
    let foreign = empty(worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap());
    assert!(super::super::prepare_sample_plan(basis, &foreign, &atlas, &mut retained).is_err());
    assert_eq!(retained.sample_override(identity), previous);
}
