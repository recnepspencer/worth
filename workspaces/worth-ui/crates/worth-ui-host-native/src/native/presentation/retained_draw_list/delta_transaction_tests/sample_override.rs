use super::*;

#[test]
fn terminal_removal_retains_exact_paint_attribution_and_rollback_restores_it() {
    let world = DrawListWorld::new();
    let initial_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mechanic = world.rect(
        initial_frame,
        world.first,
        0.0,
        UiMountedRgba8::new(20, 30, 40, 255),
    );
    let initial = world.initial(initial_frame, [mechanic]);
    let removed = UiMountedPaintOrderIdentity::for_command(initial.commands()[0].identity());
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let attribution = retained.top_paint_attribution().unwrap();
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: initial_frame,
        successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPaintCommandChange::Remove(removed.command())],
        nodes: Vec::new(),
        order: vec![UiMountedPaintOrderEdit::remove(removed)],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[]),
        damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
            mechanic.bounds(),
        )],
        auxiliary: None,
        production_cost: Default::default(),
    });

    let (_, undo) = retained.stage_delta(&delta, &[]).unwrap();
    assert_eq!(retained.order.ordered().count(), 0);
    assert_eq!(retained.top_paint_attribution(), Some(attribution));
    retained.rollback_delta(undo).unwrap();
    assert_eq!(retained.order.ordered().collect::<Vec<_>>(), vec![removed]);
    assert_eq!(retained.top_paint_attribution(), Some(attribution));
}

#[test]
fn semantic_delta_retires_invisible_sample_override_and_rollback_restores_it() {
    use worth_ui_host_contract::{
        UiMountedPresentationOpacity, UiMountedPresentationSample,
        UiMountedPresentationSampleChange, UiMountedPresentationSampleInput,
    };

    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mechanic = world.rect(
        frame,
        world.first,
        0.0,
        UiMountedRgba8::new(20, 30, 40, 255),
    );
    let initial = world.initial(frame, [mechanic]);
    let identity = initial.commands()[0].identity();
    let order = UiMountedPaintOrderIdentity::for_command(identity);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let invisible =
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            changes: vec![UiMountedPresentationSampleChange::from_runtime_sampling(
                identity,
                None,
                UiMountedPresentationOpacity::from_runtime_composition(0),
            )],
            damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
                mechanic.bounds(),
            )],
            production_cost: Default::default(),
        })
        .unwrap();
    retained.stage_sample(&invisible).unwrap();

    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: frame,
        successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPaintCommandChange::Remove(identity)],
        nodes: Vec::new(),
        order: vec![UiMountedPaintOrderEdit::remove(order)],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[]),
        damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
            mechanic.bounds(),
        )],
        auxiliary: None,
        production_cost: Default::default(),
    });

    let (_, undo) = retained.stage_delta(&delta, &[]).unwrap();
    assert!(retained.command(identity).is_none());
    assert_eq!(retained.sample_override(identity), None);
    retained.rollback_delta(undo).unwrap();
    assert_eq!(retained.command(identity), Some(&initial.commands()[0]));
    assert_eq!(
        retained.sample_override(identity),
        Some(invisible.changes()[0])
    );
}
