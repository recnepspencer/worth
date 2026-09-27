//! A thumb grab ends a settle in modal Portal content, and the Portal's exit
//! shows the content where the grab left it.
//!
//! A settle moves what the region scrolls, here the child's thumb, through
//! a Scroll layer the host holds between ticks. Grabbing the thumb retires
//! the settle's target and the track that sampled it, and holds the offset
//! where the pixels are; the frame that closes the Portal lays the thumb out
//! there. The Portal exit then samples only the Portal's Motion over the
//! layers the host holds, and none of them may still carry the retired
//! settle: the exit moves the child and its thumb exactly as it moves the
//! child's track, which no Scroll layer moves.
//!
//! The child here declares no children, so the only thing its region scrolls
//! is its thumb, which each frame lays out anew. Region content a frame keeps
//! while a settle moves it inside a modal is not built by this World.
use super::super::super::scroll_chrome_interaction::UiScrollChromePressOutcome;
use super::admitted_dismissal::advance_motion;
use super::geometry::scrollable::install_scrollable_tall_child;
use super::portal_scroll_region::open_installed;
use super::portal_scroll_settle::{child_offset, close, settling_scroll, wheel};
use super::scroll_pose_authority::block;
use super::scroll_settle_commit::LINE_EXTENT_POINTS;
use super::session::World;
use crate::mounting::presentation::platform_point_for_test;
use crate::runtime::portal::UiPortalIdentity;
use crate::runtime::scroll::chrome::UiScrollChromeAxis;
use crate::runtime::scroll::UiScrollOffset;
use worth_ui_host_contract::*;

const POINTER: u64 = 1;
const CAPTURE_EPOCH: u64 = 7;

/// The offset the host shows the child's region at, which Scroll holds, and
/// where the frame on screen presents the child's thumb.
type Held = (UiScrollOffset, UiMountedCanonicalBox);

/// The modal open over a child tall enough that its thumb travels, a notch
/// settling the child's region part way, and the child's thumb grabbed at its
/// center mid-settle; with the offset the host last showed the region at and
/// where the frame on screen presents the thumb.
fn grabbed_mid_settle() -> (World, UiPortalIdentity, Held) {
    let (mut world, portal) = open_installed(settling_scroll(), install_scrollable_tall_child);
    // The entrance plays out and the modal rests.
    advance_motion(&mut world, 11, 40);
    advance_motion(&mut world, 10_000, 41);
    wheel(&mut world, 10_001);
    // The first tick starts the settle's track; the next moves along it.
    advance_motion(&mut world, 10_100, 42);
    advance_motion(&mut world, 10_200, 43);
    assert!(
        thumb_step(&world) != 0.0,
        "the settle has moved the child's thumb part way"
    );
    let held = where_held(&world);
    let (shown, target) = (
        held.0.block_subpixels(),
        block(i64::from(LINE_EXTENT_POINTS)),
    );
    assert!(
        0 < shown && shown < target.block_subpixels(),
        "the settle is part way to its notch: {shown}"
    );
    let surface = world.surfaces[0];
    let thumb = held.1;
    let grabbed = [
        thumb.x() + thumb.width() / 2.0,
        thumb.y() + thumb.height() / 2.0,
    ];
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .expect("the page is on screen")
        .basis();
    let pressed = world
        .session
        .press_scroll_chrome(
            surface,
            platform_point_for_test(grabbed[0], grabbed[1]),
            UiHostPointerIdentity::new(POINTER),
            UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
            presentation,
        )
        .expect("a press on the thumb the Portal presents is answered");
    assert!(
        matches!(pressed, UiScrollChromePressOutcome::ThumbCaptured(_)),
        "the grab captures the child's thumb: {pressed:?}"
    );
    (world, portal, held)
}

fn where_held(world: &World) -> Held {
    let child = world.instances[4];
    let offset = child_offset(world);
    assert_eq!(
        world.session.mounted.mounted_scroll_pose(child, child),
        Some(offset),
        "the host shows the region at the offset Scroll holds"
    );
    let thumb = world
        .session
        .presented_scroll_chrome_facts(world.surfaces[0])
        .iter()
        .find(|region| region.owner_instance() == child)
        .expect("the Portal presents the child's bars")
        .facts()
        .axis(UiScrollChromeAxis::Block)
        .expect("the child's content overflows its block axis")
        .thumb();
    (offset, thumb)
}

/// How far the tick the host last showed moves the child's thumb.
fn thumb_step(world: &World) -> f32 {
    let child = world.instances[4];
    let transform = world
        .host
        .last_motion_samples()
        .into_iter()
        .find(|change| {
            change
                .command()
                .scroll_chrome_identity()
                .is_some_and(|chrome| {
                    chrome.owner_instance() == child
                        && chrome.part() == UiMountedScrollChromePart::Thumb
                })
        })
        .and_then(|change| change.transform())
        .expect("the tick shows the child's thumb");
    transform.sampled().y() - transform.source().y()
}

/// How far the tick the host last showed moves each of the child's commands
/// past the child's block track, which no Scroll layer moves: the child
/// itself, and its thumb.
fn scrolled_past_the_track(world: &World) -> Vec<[f32; 2]> {
    let child = world.instances[4];
    let steps = world
        .host
        .last_motion_samples()
        .into_iter()
        .filter(|change| {
            change.command().mounted_instance() == child
                || change
                    .command()
                    .scroll_chrome_identity()
                    .is_some_and(|chrome| chrome.owner_instance() == child)
        })
        .map(|change| {
            let transform = change.transform().expect("the exit moves the child");
            let (source, sampled) = (transform.source(), transform.sampled());
            (
                change.command(),
                [sampled.x() - source.x(), sampled.y() - source.y()],
            )
        })
        .collect::<Vec<_>>();
    let is_track = |command: &UiMountedPaintCommandIdentity| {
        command
            .scroll_chrome_identity()
            .is_some_and(|chrome| chrome.part() == UiMountedScrollChromePart::Track)
    };
    let [track_x, track_y] = steps
        .iter()
        .find(|(command, _)| is_track(command))
        .expect("the exit shows the child's track")
        .1;
    let scrolled = steps
        .iter()
        .filter(|(command, _)| !is_track(command))
        .map(|(_, [x, y])| [x - track_x, y - track_y])
        .collect::<Vec<_>>();
    assert!(
        scrolled.len() >= 2,
        "the exit shows the child and its thumb: {steps:?}"
    );
    scrolled
}

/// Close the Portal at `now` and play two ticks of its exit: the closing
/// frame shows the region where the grab `held` it, and each tick moves the
/// child and its thumb exactly as it moves the child's track.
fn exits_with_no_scroll_left(world: &mut World, portal: UiPortalIdentity, now: u64, held: Held) {
    close(world, portal, now);
    advance_motion(world, now + 1, 44);
    assert_eq!(
        where_held(world),
        held,
        "the closing frame shows the region and its thumb where the grab held them"
    );
    for (tick, epoch) in [(now + 20, 45), (now + 60, 46)] {
        advance_motion(world, tick, epoch);
        let steps = scrolled_past_the_track(world);
        assert!(
            steps.iter().all(|step| *step == [0.0, 0.0]),
            "the exit shows no Scroll layer the retired settle held: {steps:?}"
        );
    }
}

/// The grab ends the settle where the pixels are, and the frame that
/// closes the Portal lays the thumb out there.
#[test]
fn a_portal_exit_after_a_grab_shows_the_thumb_where_the_grab_left_it() {
    let (mut world, portal, held) = grabbed_mid_settle();
    exits_with_no_scroll_left(&mut world, portal, 10_201, held);
}
