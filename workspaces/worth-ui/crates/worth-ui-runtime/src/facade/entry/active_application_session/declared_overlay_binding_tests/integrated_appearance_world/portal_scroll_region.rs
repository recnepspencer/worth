//! A Scroll region inside a modal Portal scrolls where the Portal presents it.
//!
//! The Portal child lays out its region at the child's launched box, and the
//! Portal moves that child to where it opens. Its bars paint there and travel
//! with the Portal's entrance, a pointer finds them there, and a wheel there
//! scrolls that region. While the Portal is closed the child is presented
//! nowhere, and so are its bars.
use super::admitted_dismissal::advance_motion;
use super::geometry::scrollable::{install_scrollable_child, SCROLLABLE_CHILD_REGION};
use super::portal_placement_succession::{committed_bounds, presented_child};
use super::scroll_chrome_fixture::contract;
use super::session::{World, WorldScroll};
use crate::mounting::presentation::platform_point_for_test;
use crate::runtime::portal::UiPortalIdentity;
use crate::runtime::scroll::chrome::UiScrollChromePart;
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollOffset};
use worth_ui_host_contract::*;

/// How far the thumb's center sits in from the region's right edge.
const THUMB_INSET: f32 = 6.0;

/// The World with its Portal child's region scrollable, one frame published
/// while the Portal is closed, and then the Portal opened fitted to that child.
fn open() -> (World, UiPortalIdentity) {
    let declared = WorldScroll::default();
    let region = declared.region.clone().with_scroll_chrome(contract());
    open_with(WorldScroll { region, ..declared })
}

/// [`open`] with the Scroll service `declared` declares.
pub(super) fn open_with(declared: WorldScroll) -> (World, UiPortalIdentity) {
    let mut world = World::launch_with_scroll(declared);
    install_scrollable_child(&mut world.session, world.surfaces, world.instances);
    let frame = world.prepare();
    world.publish(frame, 1, true);
    assert!(
        painted(&world).is_empty(),
        "a closed Portal's content paints no bars"
    );
    assert!(
        world
            .session
            .presented_scroll_chrome_facts(world.surfaces[0])
            .iter()
            .all(|region| region.owner_instance() != world.instances[4]),
        "a closed Portal's content has no bars to press"
    );
    let portal = world.open_fitted(0, "overlay.child", 10);
    (world, portal)
}

/// The Portal child's bars the last presentation painted.
fn painted(world: &World) -> Vec<UiMountedScrollChromeMechanic> {
    world
        .host
        .last_scroll_chrome()
        .into_iter()
        .filter(|chrome| chrome.identity().owner_instance() == world.instances[4])
        .collect()
}

/// The child's region where the open Portal presents the child.
pub(super) fn presented_region(world: &World) -> [f32; 4] {
    let [x, y, ..] = presented_child(world);
    region_at([x, y])
}

/// The child's region with the child presented at `origin`.
fn region_at([x, y]: [f32; 2]) -> [f32; 4] {
    let [left, top, width, height] = SCROLLABLE_CHILD_REGION;
    [x + left, y + top, width, height]
}

/// Logical points of a mechanic coordinate.
fn points(subpixels: i64) -> f32 {
    subpixels as f32 / UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32
}

pub(super) fn subpixels(points: f32) -> i64 {
    (points * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64
}

/// Bars paint at the Portal's resting presentation, which is where the
/// child is presented once the entrance settles: a fitted Portal presents
/// its content at its own origin.
#[test]
fn portal_content_paints_its_bars_where_the_portal_presents_it() {
    let (mut world, portal) = open();
    let [x, y, ..] = committed_bounds(&world, portal);
    let region = region_at([x, y]);
    let chrome = painted(&world);
    assert_eq!(chrome.len(), 2, "one track and one thumb: {chrome:?}");
    for bar in chrome {
        let (rect, clip) = (bar.rect(), bar.clip());
        let rect = [
            rect.x().into(),
            rect.y().into(),
            rect.width().into(),
            rect.height().into(),
        ]
        .map(points);
        let clip = [
            clip.x().into(),
            clip.y().into(),
            clip.width().into(),
            clip.height().into(),
        ]
        .map(points);
        assert_eq!(
            clip, region,
            "bars clip to the region where the Portal presents it"
        );
        assert!(
            rect[0] >= region[0]
                && rect[0] + rect[2] <= region[0] + region[2]
                && rect[1] >= region[1]
                && rect[1] + rect[3] <= region[1] + region[3],
            "{rect:?} paints inside the presented region {region:?}"
        );
    }
    // The entrance starts at its first sampled tick and settles by the last.
    advance_motion(&mut world, 11, 39);
    advance_motion(&mut world, 10_000, 40);
    assert_eq!(
        presented_region(&world),
        region,
        "the settled child is presented where its bars paint"
    );
}

/// The entrance moves the bars as it moves the content they scroll.
#[test]
fn portal_content_bars_travel_with_the_portal_entrance() {
    let (mut world, _) = open();
    advance_motion(&mut world, 11, 40);
    let samples = world.host.last_motion_samples();
    let (bars, content): (Vec<&UiMountedPresentationSampleChange>, Vec<_>) =
        samples.iter().partition(|change| {
            change
                .command()
                .scroll_chrome_identity()
                .is_some_and(|chrome| chrome.owner_instance() == world.instances[4])
        });
    assert_eq!(bars.len(), 2, "the entrance samples both bars: {samples:?}");
    let moved = content
        .iter()
        .find(|change| change.command().mounted_instance() == world.instances[4])
        .expect("the entrance samples the child's content")
        .transform()
        .expect("the entrance moves the child's content");
    assert_ne!(
        moved.source().y(),
        moved.sampled().y(),
        "early in the entrance the content is still traveling"
    );
    for bar in bars {
        assert_eq!(
            bar.transform(),
            Some(moved),
            "a bar moves as the content it scrolls"
        );
    }
}

#[test]
fn a_pointer_finds_portal_content_bars_where_the_portal_presents_them() {
    let (world, _) = open();
    let region = presented_region(&world);
    let surface = world.surfaces[0];
    let thumb = [
        region[0] + region[2] - THUMB_INSET,
        region[1] + region[3] / 2.0,
    ];
    let answer = world
        .session
        .scroll_chrome_under_pointer(surface, platform_point_for_test(thumb[0], thumb[1]))
        .expect("the presented bar is under the pointer");
    assert_eq!(answer.part().unwrap().part(), UiScrollChromePart::Thumb);
    let laid_out = super::geometry::BOXES[4];
    assert!(
        world
            .session
            .scroll_chrome_under_pointer(
                surface,
                platform_point_for_test(
                    laid_out[0] + region[2] - THUMB_INSET,
                    laid_out[1] + region[3] / 2.0,
                ),
            )
            .is_none(),
        "nothing is presented where the child is laid out"
    );
}

#[test]
fn a_wheel_over_portal_content_scrolls_its_region() {
    let (mut world, _) = open();
    let region = presented_region(&world);
    let child = world.instances[4];
    let owner = world
        .session
        .scroll
        .as_ref()
        .unwrap()
        .ownership_chain(child)
        .unwrap()
        .owners()[0];
    let incarnation = world
        .session
        .mounted
        .scroll_region_incarnation(child, 0)
        .unwrap();
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap()
        .basis();
    let payload = UiHostObservationPayload::ScrollDelta {
        source: UiHostScrollDeltaSource::PointerWheel,
        phase: UiHostScrollDeltaPhase::Updated,
        precision: UiHostScrollDeltaPrecision::Pixel,
        target: UiHostScrollDeltaTargetAffinity::exact_coordinate(
            presentation,
            UiHostSurfacePosition::viewport_logical(
                subpixels(region[0] + region[2] / 2.0),
                subpixels(region[1] + region[3] / 2.0),
            ),
        ),
        x_subpixels: 0,
        // The host reports the wheel's own travel: content pushed toward
        // the top arrives negative and scrolls the region down.
        y_subpixels: -subpixels(6.0),
    };
    let outcome = world
        .session
        .observe_scroll_payload(&payload, &mut Default::default(), Some(11));
    assert!(matches!(
        outcome,
        Some(UiHostScrollObservationOutcome::Applied(_))
    ));
    // The offset is accepted once the frame that shows it is presented.
    let frame = world.prepare_surface_with_current_portals(world.surfaces[0]);
    world.publish(frame, 11, false);
    assert_eq!(
        world
            .session
            .scroll
            .as_ref()
            .unwrap()
            .offset(owner, incarnation)
            .unwrap(),
        UiScrollOffset::new(0, subpixels(6.0)).unwrap(),
        "the wheel moved the Portal child's region"
    );
}
