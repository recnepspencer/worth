//! What keeps a wheel observation off the settle path.
//!
//! `admit_smooth_wheel_observation` is the single gate: an observation becomes
//! a settle only when the declared scroll policy names a settle horizon *and*
//! the host reported the delta in lines. Either condition failing leaves the
//! observation on the direct path it took before smooth wheels existed, where
//! the host delta moves the accepted offset on the observation that carried it.
//! Both conditions are asserted here against the declarations themselves, and
//! the direct path is then exercised on the Scroll runtime state.

use super::assert_close;
use super::settle_world::{points, UiScrollSettleWorld, ONE_NOTCH_POINTS, SETTLE_TICKS};

/// An immediate declared wheel names no settle horizon, so the gate's first
/// condition cannot be met and no settle is ever staged.
#[test]
fn an_immediate_declared_wheel_offers_no_settle_horizon() {
    assert_eq!(
        crate::declaration::UiScrollPolicy::nested_region()
            .wheel_behavior()
            .settle_ticks(),
        None,
        "the default nested-region wheel is immediate"
    );
    assert_eq!(
        crate::declaration::UiScrollPolicy::nested_region()
            .with_wheel_behavior(crate::declaration::UiScrollWheelBehavior::immediate())
            .wheel_behavior()
            .settle_ticks(),
        None,
        "an explicitly immediate wheel is still immediate"
    );
    assert_eq!(
        super::settle_world::smooth_wheel_policy()
            .wheel_behavior()
            .settle_ticks(),
        Some(SETTLE_TICKS),
        "only a declared smooth wheel offers a horizon"
    );
}

/// A pixel delta carries no notch count, so the gate's second condition cannot
/// be met however the wheel was declared. Page deltas are refused for the same
/// reason.
#[test]
fn a_pixel_or_page_delta_carries_no_notch_count() {
    use worth_ui_host_contract::UiHostScrollDeltaPrecision as Precision;

    assert_eq!(Precision::Pixel.lines_per_notch(), None);
    assert_eq!(Precision::Page.lines_per_notch(), None);
    assert_eq!(
        Precision::Line {
            platform_lines_per_notch: 3,
            basis: worth_ui_host_contract::UiHostScrollLineCountBasis::PlatformReported,
        }
        .lines_per_notch(),
        Some(3),
        "only a line delta states how many lines one notch turned"
    );
}

/// The direct path is unchanged: a host delta routed against the owner's
/// reconciled bounds moves the accepted offset on the spot and stages nothing
/// for a later sample to carry.
#[test]
fn a_direct_host_delta_moves_the_accepted_offset_and_stages_no_settle() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    let entry = world.entry();
    let bounds = world.bounds();
    let request = crate::runtime::scroll::UiScrollDeltaRequest::new(
        vec![entry],
        crate::runtime::scroll::UiScrollDelta::new(0, super::settle_world::subpixels(30.0)),
        crate::runtime::scroll::UiScrollDeltaCause::Host {
            source: worth_ui_host_contract::UiHostScrollDeltaSource::PointerWheel,
            phase: worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
            precision: worth_ui_host_contract::UiHostScrollDeltaPrecision::Pixel,
        },
    )
    .expect("a single-owner chain with a block delta is a routable request");

    let receipt = world
        .scroll_mut()
        .route_with_reconciled_bounds(request, &[bounds])
        .expect("a direct host delta routes");

    assert_close(
        points(receipt.transitions()[0].current().block_subpixels()),
        30.0,
        "the direct path moves the offset on the observation that carried it",
    );
    assert_close(
        world.accepted_offset_points(),
        30.0,
        "the owner holds the moved offset immediately",
    );
    assert_eq!(
        world.pending_target(),
        None,
        "the direct path leaves nothing for a later accepted sample to carry"
    );
}

/// The two paths do not overlap. Under a smooth wheel the notch stages a target
/// and moves nothing; under the direct path the delta moves the offset and
/// stages nothing. Asserting both against the same owner keeps one from quietly
/// becoming the other.
#[test]
fn the_settle_path_and_the_direct_path_move_different_things() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    let notch = world.notch(1, 0);

    assert_eq!(notch.target_points(), ONE_NOTCH_POINTS);
    assert_eq!(
        world.accepted_offset_points(),
        0.0,
        "a staged settle has moved no accepted offset yet"
    );
    assert!(
        world.pending_target().is_some(),
        "a staged settle is a pending intention until a sample carries it"
    );
}
