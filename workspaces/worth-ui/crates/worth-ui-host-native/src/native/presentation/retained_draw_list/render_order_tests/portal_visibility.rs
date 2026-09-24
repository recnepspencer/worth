use super::*;

#[test]
fn unissued_portal_and_its_mounted_children_are_not_replayed() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let portal_mechanic = crate::native::presentation::sample::tests::portal(&world, frame);
    let portal = UiMountedPaintCommand::PortalOverlay {
        identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(
            &portal_mechanic,
        ),
        mechanic: portal_mechanic,
    };
    let child = crate::native::presentation::sample::tests::semantic_text::semantic_text_command_in_group_at(
        &world,
        frame,
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        Some(world.first),
        10,
        30.0,
    );
    let commands = [child, portal];
    let order = commands
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &commands,
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[],
    )
    .unwrap();
    let portal_surface = worth_ui_host_contract::UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        world.first,
        surface_at(portal_mechanic, u32::MAX - 4_095),
    )
    .unwrap();
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let surface_key = appearance
        .insert(
            UiNativeAppearanceCommand::PortalSurface(portal_surface),
            None,
        )
        .unwrap();
    appearance
        .insert(
            UiNativeAppearanceCommand::OverlayOrder(
                UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
                    world.surface,
                    UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
                    1,
                    1,
                    [],
                )
                .unwrap(),
            ),
            Some(surface_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let visible = retained
        .ordered_render_items(
            commands.iter().map(UiMountedPaintCommand::identity),
            [surface_key],
        )
        .unwrap();
    assert!(visible.is_empty());
}
