use super::appearance_only::surface_outline_pair_in_group;
use super::*;

#[test]
fn backdrops_bracket_appearance_only_portal_children_in_complete_and_sparse_replay() {
    use crate::native::presentation::appearance::mounted_mechanic_fixtures::backdrop_for_surface;

    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let portal_order = u32::MAX - 4_095;
    let portal_mechanic =
        crate::native::presentation::sample::tests::portal_at_order(&world, frame, portal_order);
    let portal = UiMountedPaintCommand::PortalOverlay {
        identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(
            &portal_mechanic,
        ),
        mechanic: portal_mechanic,
    };
    let child_text_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child_text = crate::native::presentation::sample::tests::semantic_text::semantic_text_command_in_group_at(
        &world,
        frame,
        child_text_instance,
        Some(world.first),
        10,
        30.0,
    );
    let commands = [child_text, portal];
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

    let first_instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let second_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let third_instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let (first, first_outline) = surface_outline_pair_in_group(
        world.rect(
            frame,
            first_instance,
            0.0,
            UiMountedRgba8::new(1, 2, 3, 255),
        ),
        4,
        Some(world.first),
    );
    let (second, _) = surface_outline_pair_in_group(
        world.rect(
            frame,
            second_instance,
            20.0,
            UiMountedRgba8::new(4, 5, 6, 255),
        ),
        8,
        Some(world.first),
    );
    let (third, _) = surface_outline_pair_in_group(
        world.rect(
            frame,
            third_instance,
            40.0,
            UiMountedRgba8::new(7, 8, 9, 255),
        ),
        12,
        Some(world.first),
    );
    let portal_surface =
        worth_ui_host_contract::UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
            world.first,
            surface_at(portal_mechanic, portal_order),
        )
        .unwrap();
    let before_backdrop = backdrop_for_surface(world.surface, 0);
    let after_backdrop = backdrop_for_surface(world.surface, 1);
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let portal_surface_key = appearance
        .insert(
            UiNativeAppearanceCommand::PortalSurface(portal_surface),
            None,
        )
        .unwrap();
    let first_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(first), None)
        .unwrap();
    let first_outline_key = appearance
        .insert(
            UiNativeAppearanceCommand::Outline(first_outline),
            Some(first_key),
        )
        .unwrap();
    let second_key = appearance
        .insert(
            UiNativeAppearanceCommand::Surface(second),
            Some(first_outline_key),
        )
        .unwrap();
    let third_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(third), Some(second_key))
        .unwrap();
    let before_backdrop_key = appearance
        .insert(
            UiNativeAppearanceCommand::Backdrop(before_backdrop.clone()),
            Some(third_key),
        )
        .unwrap();
    let after_backdrop_key = appearance
        .insert(
            UiNativeAppearanceCommand::Backdrop(after_backdrop.clone()),
            Some(before_backdrop_key),
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
                    [
                        UiOverlayParticipantIdentity::Backdrop(before_backdrop.identity().clone()),
                        UiOverlayParticipantIdentity::Portal(world.first),
                        UiOverlayParticipantIdentity::Backdrop(after_backdrop.identity().clone()),
                    ],
                )
                .unwrap(),
            ),
            Some(after_backdrop_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let complete = retained
        .ordered_render_items(
            commands.iter().map(UiMountedPaintCommand::identity),
            [
                after_backdrop_key,
                portal_surface_key,
                third_key,
                first_outline_key,
                before_backdrop_key,
                second_key,
                first_key,
            ],
        )
        .unwrap();
    assert!(matches!(
        complete.as_ref(),
        [
            UiNativeRetainedRenderItem::Appearance(before_backdrop),
            UiNativeRetainedRenderItem::Appearance(portal_surface),
            UiNativeRetainedRenderItem::Appearance(first),
            UiNativeRetainedRenderItem::Appearance(first_outline),
            UiNativeRetainedRenderItem::Appearance(second),
            UiNativeRetainedRenderItem::Paint(child_text),
            UiNativeRetainedRenderItem::Appearance(third),
            UiNativeRetainedRenderItem::Appearance(after_backdrop),
        ] if *before_backdrop == before_backdrop_key
            && *portal_surface == portal_surface_key
            && *first == first_key
            && *first_outline == first_outline_key
            && *second == second_key
            && *child_text == commands[0].identity()
            && *third == third_key
            && *after_backdrop == after_backdrop_key
    ));

    let sparse = retained
        .ordered_render_items(
            commands.iter().map(UiMountedPaintCommand::identity),
            [
                after_backdrop_key,
                portal_surface_key,
                third_key,
                first_outline_key,
                before_backdrop_key,
                first_key,
            ],
        )
        .unwrap();
    assert!(matches!(
        sparse.as_ref(),
        [
            UiNativeRetainedRenderItem::Appearance(before_backdrop),
            UiNativeRetainedRenderItem::Appearance(portal_surface),
            UiNativeRetainedRenderItem::Appearance(first),
            UiNativeRetainedRenderItem::Appearance(first_outline),
            UiNativeRetainedRenderItem::Paint(child_text),
            UiNativeRetainedRenderItem::Appearance(third),
            UiNativeRetainedRenderItem::Appearance(after_backdrop),
        ] if *before_backdrop == before_backdrop_key
            && *portal_surface == portal_surface_key
            && *first == first_key
            && *first_outline == first_outline_key
            && *child_text == commands[0].identity()
            && *third == third_key
            && *after_backdrop == after_backdrop_key
    ));
}

#[test]
fn sibling_portal_groups_follow_issued_order_when_equal_depth_mounted_order_disagrees() {
    use crate::native::presentation::appearance::mounted_mechanic_fixtures::backdrop_for_surface;

    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let second_owner = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first_portal = crate::native::presentation::sample::tests::portal_for_owner_at_order(
        &world,
        frame,
        world.first,
        7,
        u32::MAX - 4_095,
        1,
    );
    let second_portal = crate::native::presentation::sample::tests::portal_for_owner_at_order(
        &world,
        frame,
        second_owner,
        8,
        u32::MAX - 4_095,
        1,
    );
    let first_text = crate::native::presentation::sample::tests::semantic_text::semantic_text_command_in_group_at(
        &world,
        frame,
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        Some(world.first),
        8,
        10.0,
    );
    let second_text = crate::native::presentation::sample::tests::semantic_text::semantic_text_command_in_group_at(
        &world,
        frame,
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        Some(second_owner),
        8,
        50.0,
    );
    let commands = [
        first_text,
        second_text,
        // The second Portal occurs first in ordinary mounted order. The
        // issued stack below says the first Portal was opened first.
        portal_command(second_portal),
        portal_command(first_portal),
    ];
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

    let first_surface = surface_outline_pair_in_group(first_portal, 4, Some(world.first)).0;
    let second_surface = surface_outline_pair_in_group(second_portal, 4, Some(second_owner)).0;
    let first_portal_surface =
        worth_ui_host_contract::UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
            world.first,
            surface_at(first_portal, u32::MAX - 4_095),
        )
        .unwrap();
    let second_portal_surface =
        worth_ui_host_contract::UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
            second_owner,
            surface_at(second_portal, u32::MAX - 4_095),
        )
        .unwrap();
    let before = backdrop_for_surface(world.surface, 0);
    let between = backdrop_for_surface(world.surface, 1);
    let after = backdrop_for_surface(world.surface, 2);
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let first_surface_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(first_surface), None)
        .unwrap();
    let second_surface_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(second_surface), None)
        .unwrap();
    let first_portal_key = appearance
        .insert(
            UiNativeAppearanceCommand::PortalSurface(first_portal_surface),
            None,
        )
        .unwrap();
    let second_portal_key = appearance
        .insert(
            UiNativeAppearanceCommand::PortalSurface(second_portal_surface),
            None,
        )
        .unwrap();
    let before_key = appearance
        .insert(UiNativeAppearanceCommand::Backdrop(before.clone()), None)
        .unwrap();
    let between_key = appearance
        .insert(UiNativeAppearanceCommand::Backdrop(between.clone()), None)
        .unwrap();
    let after_key = appearance
        .insert(UiNativeAppearanceCommand::Backdrop(after.clone()), None)
        .unwrap();
    appearance
        .insert(
            UiNativeAppearanceCommand::OverlayOrder(
                UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
                    world.surface,
                    UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
                    1,
                    1,
                    [
                        UiOverlayParticipantIdentity::Backdrop(before.identity().clone()),
                        UiOverlayParticipantIdentity::Portal(world.first),
                        UiOverlayParticipantIdentity::Backdrop(between.identity().clone()),
                        UiOverlayParticipantIdentity::Portal(second_owner),
                        UiOverlayParticipantIdentity::Backdrop(after.identity().clone()),
                    ],
                )
                .unwrap(),
            ),
            None,
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let joined = retained
        .ordered_render_items(
            commands.iter().map(UiMountedPaintCommand::identity),
            [
                after_key,
                second_surface_key,
                first_portal_key,
                between_key,
                first_surface_key,
                second_portal_key,
                before_key,
            ],
        )
        .unwrap();
    assert!(matches!(
        joined.as_ref(),
        [
            UiNativeRetainedRenderItem::Appearance(before_item),
            UiNativeRetainedRenderItem::Appearance(first_portal_item),
            UiNativeRetainedRenderItem::Appearance(first_surface_item),
            UiNativeRetainedRenderItem::Paint(first_text_item),
            UiNativeRetainedRenderItem::Appearance(between_item),
            UiNativeRetainedRenderItem::Appearance(second_portal_item),
            UiNativeRetainedRenderItem::Appearance(second_surface_item),
            UiNativeRetainedRenderItem::Paint(second_text_item),
            UiNativeRetainedRenderItem::Appearance(after_item),
        ] if *before_item == before_key
            && *first_portal_item == first_portal_key
            && *first_surface_item == first_surface_key
            && *first_text_item == commands[0].identity()
            && *between_item == between_key
            && *second_portal_item == second_portal_key
            && *second_surface_item == second_surface_key
            && *second_text_item == commands[1].identity()
            && *after_item == after_key
    ));
}

fn portal_command(
    mechanic: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> UiMountedPaintCommand {
    UiMountedPaintCommand::PortalOverlay {
        identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(&mechanic),
        mechanic,
    }
}
