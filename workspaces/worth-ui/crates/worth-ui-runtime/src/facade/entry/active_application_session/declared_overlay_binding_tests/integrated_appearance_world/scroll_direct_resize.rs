//! A resize that arrives while direct input still awaits its frame lays out
//! from where that input put the region, and one frame accepts both.
use super::geometry::scrollable::install_scrollable_primary_with_travel;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::World;
use crate::mounting::UiMountedFrameOutcome;
use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};
use worth_ui_host_contract::{UiHostScrollDeltaPrecision, UiPresentationDeadline};

/// The nested World after one ten-point wheel whose frame is not yet shown.
fn wheeled_world() -> ScrollWorld {
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_without_motion());
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -10_000, 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(scroll.accepted_offset(), block(0));
    assert_eq!(scroll.mounted_offset(), Some(block(10)));
    scroll
}

fn resize(scroll: &mut ScrollWorld, travel: f32) {
    let (surfaces, instances) = (scroll.world.surfaces, scroll.world.instances);
    install_scrollable_primary_with_travel(
        &mut scroll.world.session,
        surfaces,
        instances,
        30,
        travel,
    );
}

fn assert_settled(scroll: &ScrollWorld, offset: crate::runtime::scroll::UiScrollOffset) {
    assert_eq!(scroll.accepted_offset(), offset);
    assert_eq!(scroll.mounted_offset(), Some(offset));
    assert!(!scroll
        .world
        .session
        .mounted
        .has_pending_direct_scroll(scroll.surface()));
    let installed = scroll.world.session.scroll.as_ref().unwrap();
    assert!(!installed.has_pending_direct(scroll.surface()));
    assert!(!installed.has_unpresented_layout(scroll.surface()));
}

#[test]
fn a_resize_before_the_wheel_frame_keeps_the_wheel_offset() {
    let mut scroll = wheeled_world();
    resize(&mut scroll, 20.0);
    assert_eq!(scroll.accepted_offset(), block(0));
    assert_eq!(scroll.mounted_offset(), Some(block(10)));
    scroll.publish_direct(6);
    assert_settled(&scroll, block(10));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_wheel_over_the_unpresented_resize_waits_for_its_frame() {
    let mut scroll = wheeled_world();
    resize(&mut scroll, 20.0);
    assert_eq!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -5_000, 6),
        UiHostScrollObservationOutcome::Denied(
            UiHostScrollObservationDenial::PendingGeometryPublication
        )
    );
    assert_eq!(scroll.mounted_offset(), Some(block(10)));
    scroll.publish_direct(7);
    assert_settled(&scroll, block(10));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn the_wheel_frame_prepared_before_the_resize_is_not_presented() {
    let mut scroll = wheeled_world();
    let stale = scroll.world.prepare_surface(scroll.surface());
    resize(&mut scroll, 20.0);
    let outcome = scroll
        .world
        .session
        .present_prepared_mounted_frame_internal(
            stale,
            UiPresentationDeadline::at_tick(u64::MAX),
            6,
        );
    assert!(
        matches!(outcome, UiMountedFrameOutcome::AdmissionDenied(_)),
        "a frame laid out before the resize is refused admission, never presented"
    );
    assert_eq!(scroll.accepted_offset(), block(0));
    scroll.publish_direct(7);
    assert_settled(&scroll, block(10));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_resize_below_the_wheel_offset_clamps_it_to_the_new_travel() {
    let mut scroll = wheeled_world();
    resize(&mut scroll, 5.0);
    assert_eq!(scroll.accepted_offset(), block(0));
    assert_eq!(scroll.mounted_offset(), Some(block(5)));
    scroll.publish_direct(6);
    assert_settled(&scroll, block(5));
    let _ = scroll.world.session.shutdown();
}
