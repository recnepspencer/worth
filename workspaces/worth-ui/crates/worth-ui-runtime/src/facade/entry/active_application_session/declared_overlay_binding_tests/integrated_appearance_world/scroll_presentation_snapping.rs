//! Facade-level proof that a fractional accepted offset presents on whole
//! device pixels without ever becoming one.
//!
//! Every other scrolling scenario in this World travels whole points, which are
//! whole device pixels at every scale the World is bound at, so none of them
//! can tell a snapped presentation from an unsnapped one. This one travels a
//! quarter of a device pixel past a whole one, which is the smallest distance
//! that makes the two readings differ, and asks each of them separately: the
//! offset Scroll holds keeps the quarter, and the box the content is painted in
//! does not have it.
//!
//! The content read is the third component, laid out inside the first
//! component's scrollable region. It is the occurrence that travels; the region
//! that owns it stays where layout put it, which is why its box is the one
//! worth reading.

use super::scroll_pose_authority::ScrollWorld;
use super::World;
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollOffset};
use worth_ui_host_contract::{
    UiHostScrollDeltaPrecision, UiMountedAllocationProjection, UiMountedInstanceIdentity,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// Whole points of travel, chosen to be a whole number of device pixels at
/// every scale a surface is bound at here, so the fraction below is the only
/// thing the grid has to deal with.
const WHOLE_POINTS: i64 = 10;

/// The block origin the third component is painted at, and the scale the
/// surface it is painted on is bound at. Reading them from the same prepared
/// frame is what makes the grid the box is checked against the grid it was
/// placed on.
fn painted_block_origin_and_scale(
    scroll: &mut ScrollWorld,
    instance: UiMountedInstanceIdentity,
    now: u64,
) -> (f32, u32) {
    let frame = scroll.world.prepare();
    let scale = frame.manifest().surfaces()[0].device_scale_milli();
    let origin = {
        let projection = frame.projection_rc_for_test();
        let allocation = projection
            .appearance_allocation_for_test(instance)
            .expect("the nested content is a projected occurrence");
        let UiMountedAllocationProjection::Known { bounds, .. } = allocation else {
            panic!("a completed occurrence projects a known allocation")
        };
        bounds.y()
    };
    scroll.world.publish(frame, now, false);
    (origin, scale)
}

/// The nearest whole device pixel to one distance in logical points.
fn snapped(logical_points: f64, scale_milli: u32) -> f64 {
    let pixels_per_point = f64::from(scale_milli) / 1_000.0;
    (logical_points * pixels_per_point).round() / pixels_per_point
}

/// A quarter of a device pixel: off the grid whichever way it is rounded, and
/// far enough from the halfway mark that which way is not in question.
fn quarter_pixel_subpixels(scale_milli: u32) -> i64 {
    let subpixels_per_pixel =
        UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT * 1_000 / i64::from(scale_milli);
    subpixels_per_pixel / 4
}

/// The reader stops a quarter of a device pixel past a whole one. Scroll holds
/// that quarter, because it is what an eased settle is made of and what hit
/// testing measures against. Presentation does not: the box the content is
/// painted in has moved a whole number of device pixels, so the text, the
/// outline and the border inside it sit on the same pixel boundaries they sat
/// on before the gesture started.
#[test]
fn a_fractional_offset_paints_on_whole_device_pixels() {
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch());
    let content = scroll.world.instances[2];
    let (rest, scale) = painted_block_origin_and_scale(&mut scroll, content, 2);

    let fraction = quarter_pixel_subpixels(scale);
    assert!(
        fraction > 0,
        "the World is bound at a scale whose pixels have a quarter: {scale}"
    );
    let travel = WHOLE_POINTS * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT + fraction;
    let outcome = scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -travel, 5);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "a fractional wheel over the region applies: {outcome:?}"
    );

    let fractional = UiScrollOffset::new(0, travel).expect("a positive offset is admissible");
    scroll.publish_direct(6);
    assert_eq!(
        scroll.accepted_offset(),
        fractional,
        "Scroll keeps every subpixel the host reported"
    );
    assert_eq!(
        scroll.displayed_offset(),
        Some(fractional),
        "and so does the pose the region applied to its content"
    );

    let (painted, _) = painted_block_origin_and_scale(&mut scroll, content, 3);
    let offset_points = travel as f64 / UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    let expected = f64::from(rest) - snapped(offset_points, scale);
    assert_eq!(
        f64::from(painted),
        expected,
        "the painted box moved by whole device pixels, not by the offset"
    );
    assert_ne!(
        f64::from(painted),
        f64::from(rest) - offset_points,
        "which is not where the unsnapped offset would have put it"
    );
    let _ = scroll.world.session.shutdown();
}
