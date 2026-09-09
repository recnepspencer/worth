use super::*;

#[test]
fn refused_identity_replacement_restores_predecessor_replay_and_allows_retry() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let predecessor = command(world.rect(
        frame,
        world.first,
        0.0,
        UiMountedRgba8::new(20, 30, 40, 255),
    ));
    let neighbor = command(world.rect(
        frame,
        world.second,
        80.0,
        UiMountedRgba8::new(50, 60, 70, 255),
    ));
    let predecessor_order = UiMountedPaintOrderIdentity::for_command(predecessor.identity());
    let neighbor_order = UiMountedPaintOrderIdentity::for_command(neighbor.identity());
    let initial_order = [predecessor_order, neighbor_order];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &[predecessor.clone(), neighbor.clone()],
        &initial_order,
        UiMountedPaintOrderIntegrity::for_order(&initial_order),
        &[],
    )
    .unwrap();
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor = command(world.rect(
        successor_frame,
        world.third,
        160.0,
        UiMountedRgba8::new(90, 100, 110, 255),
    ));
    assert_ne!(predecessor.identity(), successor.identity());
    let successor_order = UiMountedPaintOrderIdentity::for_command(successor.identity());
    let damage = [predecessor.bounds(), successor.bounds()]
        .map(UiMountedLogicalDamage::from_runtime_mounting);
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: frame,
        successor: successor_frame,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            predecessor.identity(),
            successor.clone(),
        )],
        nodes: Vec::new(),
        order: vec![
            UiMountedPaintOrderEdit::remove(predecessor_order),
            UiMountedPaintOrderEdit::place_after(successor_order, Some(neighbor_order)),
        ],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[
            neighbor_order,
            successor_order,
        ]),
        damage: damage.to_vec(),
        auxiliary: None,
        production_cost: Default::default(),
    });

    let basis = crate::native::presentation::raster::UiNativeRasterBasis::new([300, 100], 1.25);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (mut first_plan, mut undo) = retained.stage_delta(&delta, &[]).unwrap();
    retained
        .refresh_physical_delta(&delta, &mut undo, basis, &atlas, &mut first_plan)
        .unwrap();
    assert!(retained.command(predecessor.identity()).is_none());
    assert_eq!(retained.command(successor.identity()), Some(&successor));
    let refused = settle_staged_delta(
        &mut retained,
        undo,
        UiNativePresentationEffects::default(),
        Err(UiNativePresentationFailure::BeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined,
        )),
    );
    assert!(matches!(
        refused,
        Err(UiNativePresentationFailure::BeforeEffects(_))
    ));
    assert_eq!(retained.frame(), frame);
    assert_eq!(retained.command(predecessor.identity()), Some(&predecessor));
    assert_eq!(retained.command(neighbor.identity()), Some(&neighbor));
    assert!(retained.command(successor.identity()).is_none());
    assert_eq!(retained.order.ordered().collect::<Vec<_>>(), initial_order);
    assert_eq!(
        retained
            .physical_replay_for_damage(basis, [0.0, 0.0, 1.0, 1.0], &mut Default::default())
            .unwrap()
            .as_ref(),
        [predecessor.identity()]
    );
    assert!(retained
        .physical_replay_for_damage(basis, [200.0, 0.0, 1.0, 1.0], &mut Default::default())
        .unwrap()
        .is_empty());

    let (mut retry_plan, mut undo) = retained.stage_delta(&delta, &[]).unwrap();
    retained
        .refresh_physical_delta(&delta, &mut undo, basis, &atlas, &mut retry_plan)
        .unwrap();
    assert_eq!(
        retained
            .physical_replay_for_damage(basis, [200.0, 0.0, 1.0, 1.0], &mut Default::default())
            .unwrap()
            .as_ref(),
        [successor.identity()]
    );
    assert_eq!(retry_plan.regions, first_plan.regions);
    assert_eq!(retained.command(neighbor.identity()), Some(&neighbor));
    assert_eq!(retained.command(successor.identity()), Some(&successor));
    assert_eq!(
        retained.order.ordered().collect::<Vec<_>>(),
        [neighbor_order, successor_order]
    );
}
