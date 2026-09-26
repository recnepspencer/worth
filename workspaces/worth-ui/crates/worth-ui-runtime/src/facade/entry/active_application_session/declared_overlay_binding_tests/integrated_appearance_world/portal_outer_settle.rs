//! A Portal stays put while the page it opened over settles, unless the
//! settle moves what it hangs from.
//!
//! Portal content is laid out inside the page's Scroll region, so it is that
//! region's content. A centered Portal presents it at the middle of the
//! viewport wherever the page has scrolled its anchor, so a page settle still
//! under way when a modal opens moves the page's thumb and leaves the modal's
//! content where the Portal shows it. Opened from the page's content, the
//! modal's overlay stays put while the content it opened from scrolls. A menu
//! the page's own owner opens hangs from the owner's box, which the page's
//! scrolling never moves, so it stays put too.
use super::admitted_dismissal::advance_motion;
use super::scroll_chrome_fixture::contract;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, scroll_region, LINE_EXTENT_POINTS, ONE_NOTCH};
use super::session::{World, WorldScroll};
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::*;

/// A settle horizon well past the modal's entrance, so the page is still
/// settling once the modal rests.
const SETTLE_TICKS: u32 = 400;

/// The page scrollable with bars, a notch settling it, and the Portal
/// `declaration` opened over it right after: from the page's owner, or with
/// `from_content` from the page's content.
fn settling_under_an_open_portal(declaration: &str, from_content: bool) -> ScrollWorld {
    let policy = crate::declaration::UiScrollPolicy::nested_region().with_wheel_behavior(
        crate::declaration::UiScrollWheelBehavior::smooth(SETTLE_TICKS)
            .expect("a nonzero settle horizon"),
    );
    let world = World::launch_with_scroll(WorldScroll {
        policy,
        region: scroll_region(true).with_scroll_chrome(contract()),
    });
    let mut scroll = if from_content {
        ScrollWorld::publish_with_nested_content(world)
    } else {
        ScrollWorld::publish(world)
    };
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    if from_content {
        // Page content stands where the page has scrolled it, not at its
        // launched box.
        scroll.world.open_where_presented(2, declaration, None, 6);
    } else {
        scroll.world.open_fitted(0, declaration, 6);
    }
    // The entrance plays out; the page settle is still under way.
    advance_motion(&mut scroll.world, 11, 40);
    advance_motion(&mut scroll.world, 200, 41);
    scroll
}

/// The settle lands under the Portal.
fn lands(mut scroll: ScrollWorld) {
    advance_motion(&mut scroll.world, 10_000, 42);
    assert_eq!(
        scroll.accepted_offset(),
        block(i64::from(LINE_EXTENT_POINTS)),
        "the settle lands under the Portal"
    );
    let _ = scroll.world.session.shutdown();
}

/// How far the tick the host last showed moves each command `paints`
/// picks.
fn moved(world: &World, paints: impl Fn(UiMountedPaintCommandIdentity) -> bool) -> Vec<[f32; 2]> {
    world
        .host
        .last_motion_samples()
        .into_iter()
        .filter(|change| paints(change.command()))
        .filter_map(|change| change.transform())
        .map(|transform| {
            let (source, sampled) = (transform.source(), transform.sampled());
            [sampled.x() - source.x(), sampled.y() - source.y()]
        })
        .collect()
}

/// The page settle under the Portal `declaration` the page's owner opened
/// moves the page's thumb and leaves the Portal's content in place.
fn leaves_the_content_of(declaration: &str) {
    let scroll = settling_under_an_open_portal(declaration, false);
    let page = scroll.world.instances[0];
    let page_thumb = moved(&scroll.world, |command| {
        command.scroll_chrome_identity().is_some_and(|chrome| {
            chrome.owner_instance() == page && chrome.part() == UiMountedScrollChromePart::Thumb
        })
    });
    assert!(
        page_thumb.iter().any(|[_, y]| *y != 0.0),
        "the settle moves the page's thumb: {page_thumb:?}"
    );
    let portal = scroll.world.instances[4];
    let content = moved(&scroll.world, |command| {
        command.scroll_chrome_identity().is_none() && command.mounted_instance() == portal
    });
    assert!(
        !content.is_empty() && content.iter().all(|step| *step == [0.0, 0.0]),
        "the page settle leaves {declaration}'s content in place: {content:?}"
    );
    lands(scroll);
}

#[test]
fn a_page_settle_under_a_centered_modal_leaves_the_modal_content_in_place() {
    leaves_the_content_of("overlay.child");
}

#[test]
fn a_page_settle_leaves_a_menu_the_page_owner_opened_in_place() {
    leaves_the_content_of("overlay.menu");
}

#[test]
fn a_page_settle_moves_the_content_a_centered_modal_opened_from_but_not_the_modal() {
    let scroll = settling_under_an_open_portal("overlay.child", true);
    let opener = scroll.world.instances[2];
    let overlays = opened_overlays(&scroll.world, opener);
    assert!(
        !overlays.is_empty(),
        "the opener paints its modal's overlay"
    );
    let opener_paint = moved(&scroll.world, |command| {
        command.mounted_instance() == opener
            && command.scroll_chrome_identity().is_none()
            && !overlays.contains(&command)
    });
    assert!(
        opener_paint.iter().any(|[_, y]| *y != 0.0),
        "the settle moves the page content the modal opened from: {opener_paint:?}"
    );
    let overlay = moved(&scroll.world, |command| overlays.contains(&command));
    assert!(
        !overlay.is_empty() && overlay.iter().all(|step| *step == [0.0, 0.0]),
        "the page settle leaves the centered modal where the Portal shows it: {overlay:?}"
    );
    lands(scroll);
}

/// The overlays of the Portals `opener` opened, in the frame on screen.
fn opened_overlays(
    world: &World,
    opener: UiMountedInstanceIdentity,
) -> Vec<UiMountedPaintCommandIdentity> {
    let binding = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .expect("the page is on screen")
        .binding();
    world
        .session
        .current_mounted_projection_rc_for_test()
        .expect("a frame is on screen")
        .view_for(binding)
        .expect("the frame projects the page")
        .retained_paint_commands()
        .iter()
        .filter_map(|command| match command {
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } if mechanic.owner() == opener => {
                Some(command.identity())
            }
            UiMountedPaintCommand::PortalOverlay { .. }
            | UiMountedPaintCommand::SemanticText { .. } => None,
        })
        .collect()
}
