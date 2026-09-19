use super::*;

#[test]
fn denied_same_identity_replacement_restores_semantic_weight_before_retry() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let predecessor = command(world.rect_at_order(
        frame,
        world.first,
        0.0,
        UiMountedRgba8::new(20, 30, 40, 255),
        9,
    ));
    let neighbor = command(world.rect_at_order(
        frame,
        world.second,
        80.0,
        UiMountedRgba8::new(50, 60, 70, 255),
        1,
    ));
    let predecessor_order = UiMountedPaintOrderIdentity::for_command(predecessor.identity());
    let neighbor_order = UiMountedPaintOrderIdentity::for_command(neighbor.identity());
    let order = [predecessor_order, neighbor_order];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &[predecessor.clone(), neighbor],
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[],
    )
    .unwrap();

    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor = command(world.rect_at_order(
        successor_frame,
        world.first,
        0.0,
        UiMountedRgba8::new(90, 100, 110, 255),
        0,
    ));
    assert_eq!(successor.identity(), predecessor.identity());
    let make_delta = |order_integrity| {
        UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
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
            order: Vec::new(),
            order_integrity,
            damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
                predecessor.bounds(),
            )],
            auxiliary: None,
            production_cost: Default::default(),
        })
    };

    let denied = make_delta(UiMountedPaintOrderIntegrity::for_order(&[
        neighbor_order,
        predecessor_order,
    ]));
    assert!(matches!(
        retained.stage_delta(&denied, &[]),
        Err(UiNativeRetainedDrawListDenial::OrderMismatch)
    ));
    assert_eq!(
        retained.order.first_with_weight_at_least(5),
        Some(predecessor_order),
        "rollback must restore the predecessor's semantic weight"
    );
    assert_eq!(retained.command(predecessor.identity()), Some(&predecessor));

    let accepted = make_delta(UiMountedPaintOrderIntegrity::for_order(&order));
    retained.apply_delta(&accepted).unwrap();
    assert_eq!(retained.order.first_with_weight_at_least(5), None);
    assert_eq!(retained.command(successor.identity()), Some(&successor));
}
