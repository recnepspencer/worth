//! A Portal that opened fitted to its content is placed again at the extent
//! each successor layout gives that content, so a modal whose content a resize
//! lays out taller or shorter takes that extent rather than the one it opened
//! at. A Portal opened at its declared extent keeps it.
use super::geometry::{install_child_in_viewport, BOXES, VIEWPORT};
use super::portal_placement_succession::committed_bounds;
use super::session::World;

/// Centered in `viewport` inside the modal's 24-point insets, at the content's
/// extent or as much of it as the viewport leaves.
fn centered(viewport: [f32; 4], [_, _, content_width, content_height]: [f32; 4]) -> [f32; 4] {
    let [x, y, viewport_width, viewport_height] = viewport;
    let width = content_width.min(viewport_width - 48.0);
    let height = content_height.min(viewport_height - 48.0);
    [
        x + (viewport_width - width) * 0.5,
        y + (viewport_height - height) * 0.5,
        width,
        height,
    ]
}

fn lay_out(world: &mut World, revision: u64, child: [f32; 4], viewport: [f32; 4]) {
    install_child_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        revision,
        child,
        viewport,
    );
    let frame = world.prepare();
    world.publish(frame, revision, false);
}

#[test]
fn a_fitted_modal_takes_the_extent_each_layout_gives_its_content() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open_fitted(0, "overlay.child", 10);
    assert_eq!(
        committed_bounds(&world, portal),
        centered(VIEWPORT, BOXES[4]),
        "the modal opens at its content's extent"
    );

    // The content lays out larger: the modal grows to it.
    let grown = [8.0, 12.0, 300.0, 90.0];
    lay_out(&mut world, 20, grown, VIEWPORT);
    assert_eq!(committed_bounds(&world, portal), centered(VIEWPORT, grown));

    // The viewport shrinks below content that grew taller still: the modal
    // takes the height the viewport leaves.
    let smaller = [0.0, 0.0, 600.0, 360.0];
    let tall = [8.0, 12.0, 300.0, 400.0];
    lay_out(&mut world, 21, tall, smaller);
    let constrained = centered(smaller, tall);
    assert_eq!(constrained[3], 312.0, "the viewport constrains the modal");
    assert_eq!(committed_bounds(&world, portal), constrained);

    // The content returns to its opening extent: so does the modal.
    lay_out(&mut world, 22, BOXES[4], smaller);
    assert_eq!(
        committed_bounds(&world, portal),
        centered(smaller, BOXES[4])
    );
}

#[test]
fn a_modal_at_its_declared_extent_keeps_it_when_its_content_grows() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.child", None, 10);
    let declared = committed_bounds(&world, portal);
    lay_out(&mut world, 20, [8.0, 12.0, 300.0, 90.0], VIEWPORT);
    assert_eq!(committed_bounds(&world, portal), declared);
}
