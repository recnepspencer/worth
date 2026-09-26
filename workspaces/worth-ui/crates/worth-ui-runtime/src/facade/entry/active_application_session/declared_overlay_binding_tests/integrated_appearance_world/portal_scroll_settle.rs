//! A Scroll settle in modal Portal content and the Portal's own Motion share
//! the content they move.
//!
//! A wheel during the Portal's entrance composes with it. A wheel over the
//! Portal child starts a settle that outlives a close begun right after it:
//! the exit carries the content and its bars out while the settle is still
//! moving them, and once the Portal presents the child nowhere, each tick of
//! the settle still presents and the settle lands its offset.
use super::admitted_dismissal::advance_motion;
use super::portal_scroll_region::{open_with, presented_region, subpixels};
use super::scroll_chrome_fixture::contract;
use super::scroll_pose_authority::block;
use super::scroll_settle_commit::{one_notch_up, scroll_region, LINE_EXTENT_POINTS, ONE_NOTCH};
use super::session::{World, WorldScroll};
use crate::facade::entry::active_application_session::UiPortalExitTerminalProgress;
use crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome as Outcome;
use crate::mounting::UiMountedPlacement;
use crate::runtime::portal::{UiPortalIdentity, UiPortalLifecyclePosture};
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollOffset};
use worth_ui_host_contract::*;

/// A settle horizon longer than the Portal's exit, so the Portal closes
/// around a settle still under way.
const SETTLE_TICKS: u32 = 400;

/// The World with a smooth wheel over its Portal child's region, its Portal
/// open and entering.
fn opened() -> (World, UiPortalIdentity) {
    let region = scroll_region(true).with_scroll_chrome(contract());
    let policy = crate::declaration::UiScrollPolicy::nested_region().with_wheel_behavior(
        crate::declaration::UiScrollWheelBehavior::smooth(SETTLE_TICKS)
            .expect("a nonzero settle horizon"),
    );
    open_with(WorldScroll { policy, region })
}

/// One notch wheeled over the middle of where the Portal child's region
/// shows.
fn wheel(world: &mut World, now: u64) {
    let region = presented_region(world);
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap()
        .basis();
    let payload = UiHostObservationPayload::ScrollDelta {
        source: UiHostScrollDeltaSource::PointerWheel,
        phase: UiHostScrollDeltaPhase::Updated,
        precision: ONE_NOTCH,
        target: UiHostScrollDeltaTargetAffinity::exact_coordinate(
            presentation,
            UiHostSurfacePosition::viewport_logical(
                subpixels(region[0] + region[2] / 2.0),
                subpixels(region[1] + region[3] / 2.0),
            ),
        ),
        x_subpixels: 0,
        y_subpixels: one_notch_up(),
    };
    assert!(matches!(
        world
            .session
            .observe_scroll_payload(&payload, &mut Default::default(), Some(now)),
        Some(UiHostScrollObservationOutcome::Applied(_))
    ));
}

fn child_offset(world: &World) -> UiScrollOffset {
    let child = world.instances[4];
    let scroll = world.session.scroll.as_ref().unwrap();
    let owner = scroll.ownership_chain(child).unwrap().owners()[0];
    let incarnation = world
        .session
        .mounted
        .scroll_region_incarnation(child, 0)
        .unwrap();
    scroll.offset(owner, incarnation).unwrap()
}

fn close(world: &mut World, portal: UiPortalIdentity, now: u64) {
    world.host.push_native_display_presented();
    world.host.push_native_display_settled_without_effects();
    assert!(matches!(
        world
            .session
            .publish_anchor_loss_portal_dismissal(portal, now),
        Outcome::Published(_)
    ));
}

/// The Portal child's bars in the tick the host last showed.
fn child_bars(world: &World) -> Vec<UiMountedPresentationSampleChange> {
    world
        .host
        .last_motion_samples()
        .into_iter()
        .filter(|change| {
            change
                .command()
                .scroll_chrome_identity()
                .is_some_and(|chrome| chrome.owner_instance() == world.instances[4])
        })
        .collect()
}

/// The host shows the child's thumb through both Motions at once: the
/// settle moves and clips it while the Portal fades it.
fn assert_thumb_scrolled_and_faded(world: &World) {
    let thumb = child_bars(world)
        .into_iter()
        .find(|change| {
            change
                .command()
                .scroll_chrome_identity()
                .is_some_and(|chrome| chrome.part() == UiMountedScrollChromePart::Thumb)
        })
        .expect("the tick shows the child's thumb");
    assert!(thumb.clip().is_some(), "the settle clips {thumb:?}");
    assert!(
        thumb.opacity().motion_units() < u16::MAX,
        "the Portal fades {thumb:?}"
    );
}

/// One Motion tick over content the frame presents nowhere: the host shows
/// it, and it paints nothing.
fn present_unpainted_tick(world: &mut World, tick: u64) {
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let sample = world.session.prepare_motion_tick(tick, basis).unwrap();
    let calls = world.host.presentation_calls();
    world.host.push_native_display_as_issued();
    world.session.present_prepared_motion_tick(sample, basis);
    assert_eq!(
        world.host.presentation_calls(),
        calls + 1,
        "the tick presents"
    );
    assert_eq!(world.host.pending_presentation_count(), 0);
    assert!(world.host.last_motion_samples().is_empty());
    assert!(world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .is_some());
}

#[test]
fn a_wheel_during_the_entrance_composes_with_it() {
    let (mut world, _) = opened();
    advance_motion(&mut world, 11, 40);
    wheel(&mut world, 12);
    advance_motion(&mut world, 20, 41);
    assert_thumb_scrolled_and_faded(&world);
    advance_motion(&mut world, 10_000, 42);
    assert_eq!(child_offset(&world), block(i64::from(LINE_EXTENT_POINTS)));
}

#[test]
fn a_portal_closing_over_a_settle_still_plays_its_exit() {
    let (mut world, portal) = opened();
    advance_motion(&mut world, 11, 40);
    advance_motion(&mut world, 10_000, 41);
    wheel(&mut world, 10_001);
    advance_motion(&mut world, 10_002, 42);
    close(&mut world, portal, 10_003);
    // Every tick of the exit presents: neither Motion refuses the other.
    advance_motion(&mut world, 10_004, 43);
    advance_motion(&mut world, 10_040, 44);
    // Mid-exit the exit carries and fades both of the child's bars.
    let bars = child_bars(&world);
    assert_eq!(bars.len(), 2, "the exit carries both bars: {bars:?}");
    for bar in &bars {
        assert!(
            bar.opacity().motion_units() < u16::MAX,
            "the exit fades {bar:?}"
        );
    }
    assert_thumb_scrolled_and_faded(&world);
    // The exit ends; the frame the close publishes presents the child
    // nowhere.
    advance_motion(&mut world, 10_200, 45);
    world.host.push_native_display_presented();
    world.host.push_native_display_settled_without_effects();
    assert_eq!(
        world.session.progress_portal_exit_terminal(10_201),
        UiPortalExitTerminalProgress::Published
    );
    assert_eq!(
        world.session.portal.as_ref().unwrap().posture(portal),
        UiPortalLifecyclePosture::Closed
    );
    // The child owns the region the settle moves.
    assert_eq!(
        world
            .session
            .mounted
            .presented_region_placement(world.instances[4]),
        UiMountedPlacement::Hidden
    );
    // The settle still presents each tick over content shown nowhere.
    present_unpainted_tick(&mut world, 10_300);
    present_unpainted_tick(&mut world, 10_500);
    assert_eq!(child_offset(&world), block(i64::from(LINE_EXTENT_POINTS)));
}
