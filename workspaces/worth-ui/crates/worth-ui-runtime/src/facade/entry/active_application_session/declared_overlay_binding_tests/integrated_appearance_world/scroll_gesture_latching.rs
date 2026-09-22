//! Facade-level proof that a scroll gesture keeps the owner it started on.
//!
//! The scenarios all turn on one host fact: a target affinity that names only
//! the presented surface says where the gesture is but not what it belongs to.
//! Unlatched, that is ambiguous and refused. Latched, it is answerable, because
//! the gesture already named its owner and the latch is what remembers. So the
//! same payload doubles as the question "is a latch held right now?", asked
//! through the production observation entry rather than of the slot directly.
//!
//! What the gesture must survive is the pointer. A reader flicking a list does
//! not keep the pointer still, and a host that loses the coordinate mid-gesture
//! is reporting a worse target, not a different intention. What it must not
//! survive is the occurrence: an owner that was unmounted is not somewhere the
//! rest of a gesture can go.

use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, smooth_world, ONE_NOTCH, SETTLE_TICKS};
use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};
use worth_ui_host_contract::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// A host block delta that scrolls content toward the top of the viewport by
/// `points`, which moves the offset the other way.
fn pixels(points: i64) -> i64 {
    -points * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
}

/// Whether a wheel naming only the presented surface reaches an owner. It does
/// exactly when a latch is held, so this reads back the latch through the
/// production path.
fn surface_only_wheel_reaches_an_owner(
    scroll: &mut ScrollWorld,
    phase: UiHostScrollDeltaPhase,
    precision: UiHostScrollDeltaPrecision,
    y_subpixels: i64,
    tick: u64,
) -> bool {
    let target = scroll.surface_only_target();
    match scroll.targeted_wheel(phase, target, precision, y_subpixels, tick) {
        UiHostScrollObservationOutcome::Applied(_) => true,
        UiHostScrollObservationOutcome::Denied(
            UiHostScrollObservationDenial::PresentedSurfaceFallbackIsAmbiguous,
        ) => false,
        other => panic!("a surface-only wheel either reaches its latch or is ambiguous: {other:?}"),
    }
}

/// A phased gesture states its own beginning and end. Between them the owner it
/// first moved keeps it, even once the host can no longer say where the pointer
/// is; after the end, the same report is ambiguous again.
#[test]
fn a_phased_gesture_keeps_its_owner_until_the_host_ends_it() {
    let mut scroll = ScrollWorld::launch_published();
    let target = scroll.pointer_target();
    assert!(matches!(
        scroll.targeted_wheel(
            UiHostScrollDeltaPhase::Started,
            target,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(20),
            5,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    scroll.publish_direct(5);
    assert_eq!(scroll.accepted_offset(), block(20));

    assert!(
        surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(5),
            6,
        ),
        "the gesture that moved this owner still owns the rest of itself"
    );
    // At offset 20 the painted child has already left the viewport.
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 6, false);
    assert_eq!(
        scroll.accepted_offset(),
        block(25),
        "the latched owner is the one that moved, not a surface fallback"
    );

    assert!(surface_only_wheel_reaches_an_owner(
        &mut scroll,
        UiHostScrollDeltaPhase::Ended,
        UiHostScrollDeltaPrecision::Pixel,
        pixels(5),
        7,
    ));
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 7, false);
    assert_eq!(scroll.accepted_offset(), block(30));

    assert!(
        !surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(5),
            8,
        ),
        "the gesture ended, so a report naming only the surface names nobody"
    );
    assert_eq!(scroll.accepted_offset(), block(30));
    let _ = scroll.world.session.shutdown();
}

/// A cancelled gesture ends its latch the same way an ended one does. It also
/// moves no offset, so the cancellation is visible twice over.
#[test]
fn a_cancelled_gesture_releases_its_owner_and_moves_nothing() {
    let mut scroll = ScrollWorld::launch_published();
    let target = scroll.pointer_target();
    assert!(matches!(
        scroll.targeted_wheel(
            UiHostScrollDeltaPhase::Started,
            target,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(20),
            5,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));

    assert!(surface_only_wheel_reaches_an_owner(
        &mut scroll,
        UiHostScrollDeltaPhase::Cancelled,
        UiHostScrollDeltaPrecision::Pixel,
        pixels(40),
        6,
    ));
    scroll.publish_direct(6);
    assert_eq!(
        scroll.accepted_offset(),
        block(20),
        "a cancelled phase routes no travel"
    );
    assert!(
        !surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(5),
            7,
        ),
        "a cancelled gesture is over"
    );
    let _ = scroll.world.session.shutdown();
}

/// A gesture's end is the host's report, not the runtime's conclusion, so the
/// latch it releases has to go whether or not the report was allowed to do
/// anything else. Routing is the first thing a scroll observation does and the
/// place a frame the reader has already left is refused, so an end arriving
/// against one never reaches the committing work. A phased latch has no quiet
/// interval to expire on, so one left held is held for the rest of the session,
/// and the next report the host cannot place would be handed to the owner this
/// gesture left behind.
#[test]
fn a_refused_end_still_releases_the_gesture_it_ended() {
    let mut scroll = ScrollWorld::launch_published();
    let target = scroll.pointer_target();
    assert!(matches!(
        scroll.targeted_wheel(
            UiHostScrollDeltaPhase::Started,
            target,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(20),
            5,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    scroll.publish_direct(5);
    assert_eq!(scroll.accepted_offset(), block(20));

    // The frame the gesture began on, held while the surface is published
    // again underneath it. What the host sends next is a report from a frame
    // that is no longer on screen.
    let left_behind = scroll.surface_only_target();
    let frame = scroll.world.prepare();
    scroll.world.publish(frame, 6, false);

    let outcome = scroll.targeted_wheel(
        UiHostScrollDeltaPhase::Ended,
        left_behind,
        UiHostScrollDeltaPrecision::Pixel,
        pixels(5),
        7,
    );
    assert!(
        matches!(
            outcome,
            UiHostScrollObservationOutcome::Denied(UiHostScrollObservationDenial::Targeting(_))
        ),
        "the end named a presentation the surface has replaced: {outcome:?}"
    );
    assert_eq!(
        scroll.accepted_offset(),
        block(20),
        "a refused report routes nothing"
    );

    assert!(
        !surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(5),
            8,
        ),
        "the gesture is over, however little its ending report was allowed to do"
    );
    assert_eq!(
        scroll.accepted_offset(),
        block(20),
        "and nothing reached the owner it used to name"
    );
    let _ = scroll.world.session.shutdown();
}

/// A gesture no owner had room for named nobody, so there is nothing to latch.
/// Pushing content further up while the offset already rests at zero is that
/// gesture: it is admitted, it moves nothing, and it leaves the next report to
/// the pointer.
#[test]
fn a_gesture_that_no_owner_had_room_for_takes_no_latch() {
    let mut scroll = ScrollWorld::launch_published();
    let target = scroll.pointer_target();
    assert!(matches!(
        scroll.targeted_wheel(
            UiHostScrollDeltaPhase::Started,
            target,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(-20),
            5,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(scroll.accepted_offset(), block(0));
    assert!(
        !surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(5),
            6,
        ),
        "an owner that never moved never took the gesture"
    );
    let _ = scroll.world.session.shutdown();
}

/// An unphased coarse wheel never ends; it goes quiet. Its latch therefore
/// outlives each notch by the declared settle horizon and no longer: while the
/// content that notch started is still moving, the same owner keeps the wheel,
/// and once it is still the pointer decides again.
#[test]
fn an_unphased_wheel_keeps_its_owner_for_the_declared_quiet_interval() {
    let mut scroll = smooth_world(true);
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));

    let last_live = 5 + u64::from(SETTLE_TICKS);
    assert!(
        surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            ONE_NOTCH,
            one_notch_up(),
            last_live,
        ),
        "a notch inside the interval belongs to the owner that is still settling"
    );

    // That notch carried the latch forward, so the interval is measured from
    // it rather than from the gesture's first event.
    assert!(
        !surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            ONE_NOTCH,
            one_notch_up(),
            last_live + u64::from(SETTLE_TICKS) + 1,
        ),
        "once the content is still, the pointer decides again"
    );
    let _ = scroll.world.session.shutdown();
}

/// A latch names an occurrence, not a place on screen. Unmounting the
/// occurrence leaves the latch naming nothing, and the gesture falls back to
/// the pointer rather than reaching a region that is gone.
#[test]
fn a_latch_does_not_survive_the_occurrence_it_named() {
    let mut scroll = ScrollWorld::launch_published();
    let target = scroll.pointer_target();
    assert!(matches!(
        scroll.targeted_wheel(
            UiHostScrollDeltaPhase::Started,
            target,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(20),
            5,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert!(surface_only_wheel_reaches_an_owner(
        &mut scroll,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaPrecision::Pixel,
        pixels(5),
        6,
    ));

    let occurrence = scroll.target();
    scroll
        .world
        .session
        .unmount_instance_with_interaction_receipt(occurrence)
        .expect("a mounted occurrence unmounts");
    assert!(
        !surface_only_wheel_reaches_an_owner(
            &mut scroll,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaPrecision::Pixel,
            pixels(5),
            7,
        ),
        "the region the gesture was scrolling is gone"
    );
    let _ = scroll.world.session.shutdown();
}
