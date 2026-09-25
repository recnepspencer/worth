//! An open Portal is placed again by every frame that moves its anchor or
//! resizes its viewport, with the same fit, flip, and clamp arithmetic that
//! opened it. The Portal commits the new placement only with the accepted
//! frame, so dismissal geometry always matches the pixels on screen.
use super::geometry::{install_owner_in_viewport, BOXES, MOVED_TARGET_BOX, VIEWPORT};
use super::session::World;

/// The dropdown's declared box: 280 by 320, 8 points from its anchor and 16
/// inside the viewport.
const OPENED: [f32; 4] = [40.0, 118.0, 280.0, 320.0];

pub(super) fn committed_bounds(
    world: &World,
    portal: crate::runtime::portal::UiPortalIdentity,
) -> [f32; 4] {
    let placement = world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(portal)
        .unwrap()
        .prepared();
    placement.bounds().components()
}

pub(super) fn presented_child(world: &World) -> [f32; 4] {
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let row = *world
        .session
        .mounted
        .interaction_hit_test_basis(presentation.basis())
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == world.instances[4])
        .expect("the Portal child is presented and hit-testable");
    let bounds = row.bounds().platform_box();
    [bounds.x(), bounds.y(), bounds.width(), bounds.height()]
}

/// Where the child presents inside its Portal. Its owner-relative offset
/// carries the entrance Motion's current travel too, which no frame here
/// advances.
fn child_inset(world: &World, portal: crate::runtime::portal::UiPortalIdentity) -> [f32; 2] {
    let [x, y, _, _] = committed_bounds(world, portal);
    let [child_x, child_y, width, height] = presented_child(world);
    assert_eq!([width, height], [BOXES[4][2], BOXES[4][3]]);
    [child_x - x, child_y - y]
}

fn assert_presented_at(
    world: &World,
    portal: crate::runtime::portal::UiPortalIdentity,
    bounds: [f32; 4],
    inset: [f32; 2],
) {
    assert_eq!(
        committed_bounds(world, portal),
        bounds,
        "committed placement"
    );
    assert_eq!(
        child_inset(world, portal),
        inset,
        "the child presents where the Portal is committed"
    );
}

#[test]
fn an_open_popover_follows_its_anchor_and_flips_and_clamps_in_the_new_viewport() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);
    assert_eq!(committed_bounds(&world, portal), OPENED);
    let inset = child_inset(&world, portal);
    assert_eq!(
        inset[0], BOXES[4][0],
        "the child keeps its owner-relative inset"
    );

    // The anchor moves near the bottom right: the popover flips above it and
    // clamps inside the right margin.
    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        20,
        MOVED_TARGET_BOX,
        VIEWPORT,
    );
    let moved = world.prepare();
    world.publish(moved, 20, false);
    assert_presented_at(&world, portal, [504.0, 92.0, 280.0, 320.0], inset);

    // The anchor returns and the viewport shrinks: the popover stays below
    // its anchor, clamped to the height left above the bottom margin.
    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        21,
        BOXES[0],
        [0.0, 0.0, 600.0, 400.0],
    );
    let resized = world.prepare();
    world.publish(resized, 21, false);
    assert_presented_at(&world, portal, [40.0, 118.0, 280.0, 266.0], inset);
}

/// Centered in the viewport inside its 24-point insets, at its declared
/// width and at most the height the viewport leaves.
fn centered_in(viewport: [f32; 4], [_, _, width, height]: [f32; 4]) -> [f32; 4] {
    let [x, y, viewport_width, viewport_height] = viewport;
    let height = height.min(viewport_height - 48.0);
    [
        x + (viewport_width - width) * 0.5,
        y + (viewport_height - height) * 0.5,
        width,
        height,
    ]
}

#[test]
fn an_open_modal_stays_centered_when_its_anchor_moves_and_the_viewport_shrinks() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.child", None, 10);
    let opened = committed_bounds(&world, portal);
    assert_eq!(opened, centered_in(VIEWPORT, opened));
    let inset = child_inset(&world, portal);

    // Moving the anchor does not move a modal.
    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        20,
        MOVED_TARGET_BOX,
        VIEWPORT,
    );
    let moved = world.prepare();
    world.publish(moved, 20, false);
    assert_presented_at(&world, portal, opened, inset);

    // A smaller viewport centers it again, shortened to fit its insets.
    let smaller = [0.0, 0.0, 600.0, 360.0];
    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        21,
        MOVED_TARGET_BOX,
        smaller,
    );
    let resized = world.prepare();
    world.publish(resized, 21, false);
    let recentered = centered_in(smaller, opened);
    assert!(
        recentered[3] < opened[3],
        "the viewport constrains the modal"
    );
    assert_presented_at(&world, portal, recentered, inset);
}

#[test]
fn a_frame_that_moves_nothing_or_cannot_place_the_popover_keeps_its_placement() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);
    let inset = child_inset(&world, portal);

    let unchanged = world.prepare();
    world.publish(unchanged, 20, false);
    assert_presented_at(&world, portal, OPENED, inset);

    // A viewport narrower than the margins leaves no room to place it: the
    // popover stays where it was rather than taking an invented box.
    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        21,
        BOXES[0],
        [0.0, 0.0, 24.0, 24.0],
    );
    let cramped = world.prepare();
    world.publish(cramped, 21, false);
    assert_eq!(committed_bounds(&world, portal), OPENED);
}

/// A submenu opens from content inside its parent Portal, so its anchor is
/// where that parent presents the content: the owner's layout moved by the
/// parent's placement. The parent's entrance Motion is presentation, not
/// layout, so it does not move the anchor.
#[test]
fn a_nested_popover_follows_its_owner_through_its_parents_new_placement() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    assert_eq!(committed_bounds(&world, parent), OPENED);
    let child = world.open_where_presented(4, "overlay.menu", Some(parent), 11);
    // The owner's box inside the parent, [8, 12, 140, 36], sits at [48, 130]
    // under the parent placed at [40, 118]; the submenu opens 8 below it.
    assert_eq!(committed_bounds(&world, child), [48.0, 174.0, 280.0, 320.0]);

    // The parent flips above its moved anchor and clamps to the right margin,
    // carrying the owner to [512, 104]; the submenu follows and clamps too.
    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        20,
        MOVED_TARGET_BOX,
        VIEWPORT,
    );
    let moved = world.prepare();
    world.publish(moved, 20, false);
    assert_eq!(
        committed_bounds(&world, parent),
        [504.0, 92.0, 280.0, 320.0]
    );
    assert_eq!(
        committed_bounds(&world, child),
        [504.0, 148.0, 280.0, 320.0]
    );
}

/// Content inside a scrolled region is laid out where the accepted offset put
/// it, and a popover anchored there follows it by the same distance.
#[test]
fn a_popover_on_scrolled_content_follows_the_scroll() {
    use super::scroll_pose_authority::{block, ScrollWorld};
    use worth_ui_host_contract::UiHostScrollDeltaPrecision;

    let mut scroll = ScrollWorld::launch_published();
    let portal = scroll
        .world
        .open_where_presented(2, "overlay.menu", None, 10);
    // The nested content presents at [48, 62, 160, 8]; the popover opens 8 below.
    assert_eq!(
        committed_bounds(&scroll.world, portal),
        [48.0, 78.0, 280.0, 320.0]
    );

    assert!(matches!(
        scroll.wheel(
            UiHostScrollDeltaPrecision::Pixel,
            -5 * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
            11,
        ),
        crate::runtime::scroll::UiHostScrollObservationOutcome::Applied(_)
    ));
    let surface = scroll.surface();
    let scrolled = scroll.world.prepare_surface_with_current_portals(surface);
    scroll.world.publish(scrolled, 11, false);
    assert_eq!(scroll.accepted_offset(), block(5));
    assert_eq!(
        committed_bounds(&scroll.world, portal),
        [48.0, 73.0, 280.0, 320.0]
    );
}

/// A frame fitted for an earlier open of the same Portal must not move the
/// Portal as it is open now: the published placement is adopted only for the
/// open it was fitted for, which the stack ordinal names.
#[test]
fn a_placement_fitted_for_another_open_of_the_portal_is_not_adopted() {
    use crate::runtime::portal::UiPortalStackOrdinal;

    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);

    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        20,
        MOVED_TARGET_BOX,
        VIEWPORT,
    );
    let moved = world.prepare();
    world.publish(moved, 20, false);
    let fitted = world
        .session
        .current_mounted_publication()
        .unwrap()
        .portal_placements()
        .to_vec();
    let [(ordinal, moved_placement)] = fitted[..] else {
        panic!("the moved frame places exactly the one open Portal");
    };

    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        21,
        BOXES[0],
        VIEWPORT,
    );
    let restored = world.prepare();
    world.publish(restored, 21, false);
    assert_eq!(committed_bounds(&world, portal), OPENED);

    let rebind = |world: &mut World, ordinal: UiPortalStackOrdinal| {
        let publication = world.session.current_mounted_publication().unwrap().clone();
        let portal_state = world.session.portal.as_mut().unwrap();
        publication.with_surface_presentations(|surfaces| {
            portal_state.rebind_published_presentations(
                publication.frame(),
                surfaces,
                &[(ordinal, moved_placement)],
            );
        });
    };
    rebind(
        &mut world,
        UiPortalStackOrdinal::minted_for_test(ordinal.value() + 1),
    );
    assert_eq!(committed_bounds(&world, portal), OPENED);
    rebind(&mut world, ordinal);
    assert_eq!(
        committed_bounds(&world, portal),
        moved_placement.bounds().components()
    );
}
