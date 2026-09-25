//! Facade-level proof that pressing a moving thumb takes the region over
//! without moving it, and that the drag which follows places what the pointer
//! names.
//!
//! A reader who reaches for a thumb while the content is still easing toward a
//! wheel notch is asking for two things at once. Everything still moving the
//! region has to end -- the pending wheel target, the retained accepted sample
//! and the Motion track walking it -- because a settle that survived the grab
//! would pull the content out from under it one frame later. And nothing may
//! move at the moment of capture: a press that finished the settle by jumping
//! to its target, or that abandoned it by snapping back to where it began,
//! would be a jump the reader did not ask for and could not have seen coming.
//!
//! The World here declares the chrome it presses, so every step is the
//! production one: chrome derived from the accepted pose, a pointer resolved
//! against the derived rectangles, a press that latches and captures, and a
//! move that places an absolute offset through Scroll's own route. What the
//! drag places is checked against the proportion the track and the thumb
//! themselves state, so the expectation is arithmetic rather than a number
//! copied off a run.

use super::super::super::scroll_chrome_interaction::UiScrollChromePressOutcome;
use super::super::super::UiScrollSettleDisposition;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_scroll, LINE_EXTENT_POINTS, ONE_NOTCH,
};
use super::scroll_settle_frame::{owed_frame, settle_frame};
use super::session::World;
use crate::runtime::scroll::chrome::UiScrollChromeAxis;
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollOffset};
use worth_ui_host_contract::*;

#[path = "scroll_capture_cancellation/accepted_sample_direct.rs"]
mod accepted_sample_direct;
#[path = "scroll_capture_cancellation/pending_focus_loss.rs"]
mod pending_focus_loss;
#[path = "scroll_capture_cancellation/pending_sample.rs"]
mod pending_sample;
#[path = "scroll_capture_cancellation/release_position.rs"]
mod release_position;

/// When the notch arrives, and the pointer and capture the host opens for the
/// grab that interrupts it.
const NOTCH_TICK: u64 = 5;
const POINTER: u64 = 1;
const CAPTURE_EPOCH: u64 = 7;

/// How far the content this region carries can travel.
const CONTENT_TRAVEL_POINTS: i64 = 30;
/// How far the thumb can travel: the block track is as long as the viewport,
/// and the thumb is held at its minimum length inside it. Read back off the
/// presented rectangles before it is relied on.
const THUMB_TRAVEL_POINTS: u8 = 6;
/// How far the pointer drags the thumb, and the content travel that answers
/// it. The two are tied together by the proportion the track states, which the
/// scenario checks rather than assumes.
const DRAG_POINTS: u8 = 2;
const DRAGGED_CONTENT_POINTS: i64 = 10;

/// The World the scrolling scenarios share, with a smooth wheel over a region
/// that presents chrome.
fn chrome_world() -> ScrollWorld {
    let mut declared = smooth_scroll(true);
    declared.region = declared
        .region
        .clone()
        .with_scroll_chrome(super::scroll_chrome_fixture::contract());
    ScrollWorld::publish(World::launch_with_scroll(declared))
}

/// The block track and thumb the scrollable region presents at its accepted
/// pose.
fn block_chrome(scroll: &ScrollWorld) -> (UiMountedCanonicalBox, UiMountedCanonicalBox) {
    let surface = scroll.surface();
    let regions = scroll.world.session.presented_scroll_chrome_facts(surface);
    let region = regions
        .iter()
        .find(|region| region.owner() == scroll.owner)
        .expect("the scrollable region declares chrome");
    let axis = region
        .facts()
        .axis(UiScrollChromeAxis::Block)
        .expect("the block axis is the one whose content overflows");
    (axis.track(), axis.thumb())
}

fn centre(box_: UiMountedCanonicalBox) -> [f32; 2] {
    [
        box_.x() + box_.width() / 2.0,
        box_.y() + box_.height() / 2.0,
    ]
}

/// A press the host delivers with its pointer and the capture it opened.
fn press(scroll: &mut ScrollWorld, point: [f32; 2]) -> UiScrollChromePressOutcome {
    let surface = scroll.surface();
    let presentation = scroll.presentation();
    scroll
        .world
        .session
        .press_scroll_chrome(
            surface,
            crate::mounting::presentation::platform_point_for_test(point[0], point[1]),
            UiHostPointerIdentity::new(POINTER),
            UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
            presentation,
        )
        .expect("a press on the thumb the region presents is answered")
}

/// The offset a move under the same capture places.
fn drag(scroll: &mut ScrollWorld, point: [f32; 2]) -> UiScrollOffset {
    scroll
        .world
        .session
        .drag_scroll_chrome(
            crate::mounting::presentation::platform_point_for_test(point[0], point[1]),
            UiHostPointerIdentity::new(POINTER),
            UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
        )
        .expect("a latched drag places its offset")
        .transitions()[0]
        .current()
}

fn active_motion_tracks(scroll: &ScrollWorld) -> u16 {
    scroll
        .world
        .session
        .runtime_service_resource_census()
        .active_motion_tracks()
}

/// A notch, then two of the frames the settle it published owes, so the
/// content is somewhere between where it started and where the notch is taking
/// it. Answers the offset the pointer will find when it arrives.
fn easing(scroll: &mut ScrollWorld) -> UiScrollOffset {
    assert!(
        matches!(
            scroll.wheel(ONE_NOTCH, one_notch_up(), NOTCH_TICK),
            UiHostScrollObservationOutcome::Applied(_)
        ),
        "a smooth notch over the region publishes its settle"
    );
    for elapsed in 1..=2 {
        assert_eq!(
            settle_frame(scroll, NOTCH_TICK + elapsed),
            UiScrollSettleDisposition::Applied
        );
    }
    let held = scroll.accepted_offset();
    assert!(
        held.block_subpixels() > 0
            && held.block_subpixels() < block(i64::from(LINE_EXTENT_POINTS)).block_subpixels(),
        "the settle is part way there, so the capture has something to interrupt \
         and somewhere of its own to stand: {held:?}"
    );
    assert_eq!(pending_transitions(scroll), 1);
    assert!(scroll.world.session.mounted.has_active_motion_samples());
    assert_eq!(active_motion_tracks(scroll), 1);
    held
}

/// Pressing the thumb of a region a settle is still walking ends the settle
/// and leaves the content exactly where the press found it.
#[test]
fn a_thumb_pressed_during_a_settle_ends_it_where_the_pixels_already_are() {
    let mut scroll = chrome_world();
    let held = easing(&mut scroll);
    let (_, thumb) = block_chrome(&scroll);

    let pressed = press(&mut scroll, centre(thumb));
    assert!(
        matches!(pressed, UiScrollChromePressOutcome::ThumbCaptured(_)),
        "the centre of the thumb is the thumb, not the track behind it"
    );

    assert_eq!(
        scroll.accepted_offset(),
        held,
        "the capture moved nothing: the offset is the one the settle had reached"
    );
    assert_eq!(
        scroll.mounted_offset(),
        Some(held),
        "and the pose the reader is looking at is that same offset"
    );
    assert_eq!(
        pending_transitions(&scroll),
        0,
        "the wheel target the settle was walking toward is the pointer's to \
         replace, so it is gone rather than waiting"
    );
    assert!(
        !scroll.world.session.mounted.has_active_motion_samples(),
        "no accepted sample stands behind the pose any more"
    );
    assert_eq!(
        active_motion_tracks(&scroll),
        0,
        "and no track is left walking the region the pointer now owns"
    );

    assert_eq!(
        owed_frame(&mut scroll),
        UiScrollSettleDisposition::Idle,
        "so the frame after the grab has nothing to settle"
    );
    assert_eq!(
        scroll.accepted_offset(),
        held,
        "and the content stays where the capture left it"
    );
    let _ = scroll.world.session.shutdown();
}

/// The drag that follows the capture places the offset the thumb's new
/// position names, counted from the offset the capture held, and the thumb
/// keeps the spot the pointer grabbed it by.
#[test]
fn a_drag_after_the_capture_places_the_offset_its_thumb_position_names() {
    let mut scroll = chrome_world();
    let held = easing(&mut scroll);
    let (track, thumb) = block_chrome(&scroll);
    assert_eq!(
        track.height() - thumb.height(),
        f32::from(THUMB_TRAVEL_POINTS),
        "the thumb has this much of its track left to travel in"
    );
    assert_eq!(
        DRAGGED_CONTENT_POINTS * i64::from(THUMB_TRAVEL_POINTS),
        CONTENT_TRAVEL_POINTS * i64::from(DRAG_POINTS),
        "so a drag of that many points is a content travel of this many: the \
         content covers its travel in the travel the thumb has"
    );

    let grabbed = centre(thumb);
    assert!(matches!(
        press(&mut scroll, grabbed),
        UiScrollChromePressOutcome::ThumbCaptured(_)
    ));
    let placed = drag(
        &mut scroll,
        [grabbed[0], grabbed[1] + f32::from(DRAG_POINTS)],
    );

    let expected = UiScrollOffset::new(
        0,
        held.block_subpixels() + block(DRAGGED_CONTENT_POINTS).block_subpixels(),
    )
    .expect("an in-range offset");
    assert_eq!(
        placed, expected,
        "the drag placed what its thumb position names, counted from the offset \
         the capture held rather than from where the settle was aiming"
    );
    assert_eq!(scroll.accepted_offset(), held);
    scroll.publish_direct(NOTCH_TICK + 3);
    assert_eq!(scroll.accepted_offset(), placed);
    assert_eq!(
        scroll.mounted_offset(),
        Some(placed),
        "and the pose geometry displays is the offset that was placed"
    );

    let (_, moved) = block_chrome(&scroll);
    assert_eq!(
        moved.y() - thumb.y(),
        f32::from(DRAG_POINTS),
        "the thumb travelled exactly as far as the pointer did, so the spot the \
         reader grabbed is still under it"
    );
    let _ = scroll.world.session.shutdown();
}

/// A press on a region at rest ends nothing, so the retirements the first
/// scenario reads are the settle being taken over rather than a press that
/// clears the same state whatever it finds.
#[test]
fn a_thumb_pressed_at_rest_has_nothing_to_end() {
    let mut scroll = chrome_world();
    let (_, thumb) = block_chrome(&scroll);
    assert_eq!(pending_transitions(&scroll), 0);
    assert_eq!(active_motion_tracks(&scroll), 0);

    assert!(matches!(
        press(&mut scroll, centre(thumb)),
        UiScrollChromePressOutcome::ThumbCaptured(_)
    ));
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "the press placed nothing"
    );
    assert_eq!(pending_transitions(&scroll), 0);
    assert_eq!(active_motion_tracks(&scroll), 0);
    let _ = scroll.world.session.shutdown();
}
