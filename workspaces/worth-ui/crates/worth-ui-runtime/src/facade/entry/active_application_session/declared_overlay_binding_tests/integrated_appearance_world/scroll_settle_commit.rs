//! Facade-level proof that a smooth wheel notch commits Scroll state only
//! when its settle is published.
//!
//! A smooth notch moves no accepted offset itself: it stages a settle target
//! and publishes the Motion transition that will carry the content there. The
//! staged target and the routed chain live on a successor state until that
//! publication succeeds, so a notch whose settle cannot publish leaves no
//! target behind and no offset moved.
//!
//! A settle with no Motion owner to publish to is not a case a launched
//! session can reach: policy normalization refuses a smooth wheel behaviour
//! unless the application also declares Motion, so that refusal is proved at
//! the declaration boundary rather than here.

use super::scroll_pose_authority::{block, ScrollWorld};
use super::session::{World, WorldScroll};
use crate::runtime::scroll::{
    UiHostScrollObservationDenial, UiHostScrollObservationOutcome, UiScrollSettleStop,
};
use worth_ui_host_contract::*;

pub(super) const SETTLE_TICKS: u32 = 6;
pub(super) const LINE_EXTENT_POINTS: u16 = 10;
pub(super) const ONE_NOTCH: UiHostScrollDeltaPrecision = UiHostScrollDeltaPrecision::Line {
    platform_lines_per_notch: 1,
    basis: UiHostScrollLineCountBasis::PlatformReported,
};

/// The World with a smooth wheel and a primary region that does or does not
/// declare a line extent. A painted descendant makes every physical sample
/// observable; an empty region cannot prove host-accepted content movement.
pub(super) fn smooth_world(line_extent: bool) -> ScrollWorld {
    ScrollWorld::publish_with_nested_content(World::launch_with_scroll(smooth_scroll(line_extent)))
}

/// What a smooth-wheel World declares about scrolling, so a scenario that
/// needs a different geometry can launch the same Scroll service.
pub(super) fn smooth_scroll(line_extent: bool) -> WorldScroll {
    WorldScroll {
        policy: crate::declaration::UiScrollPolicy::nested_region().with_wheel_behavior(
            crate::declaration::UiScrollWheelBehavior::smooth(SETTLE_TICKS)
                .expect("a nonzero settle horizon"),
        ),
        region: scroll_region(line_extent),
    }
}

/// The primary region that does or does not declare how far one of its lines
/// reaches. The wheel behaviour declared over it belongs to the scenario; the
/// extent a notch is measured against is shared, so a smooth notch and an
/// immediate notch answer to the same declaration.
pub(super) fn scroll_region(line_extent: bool) -> crate::capability::MosaicRegionKindDescriptor {
    let mut region =
        crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region();
    if line_extent {
        region = region.with_scroll_line_extent(
            crate::capability::UiScrollLineExtent::logical_points(LINE_EXTENT_POINTS)
                .expect("a nonzero line extent"),
        );
    }
    region
}

/// One notch toward the top of the content: a milli-line count in offset
/// direction once the host sign is turned.
pub(super) fn one_notch_up() -> i64 {
    -worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
}

pub(super) fn pending_transitions(scroll: &ScrollWorld) -> usize {
    scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .pending_transition_count()
}

#[test]
fn a_published_smooth_settle_stages_its_target_and_moves_no_offset() {
    let mut scroll = smooth_world(true);
    let displayed_before = scroll.displayed_offset();

    let UiHostScrollObservationOutcome::Applied(receipt) =
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5)
    else {
        panic!("a smooth notch with a Motion service publishes its settle")
    };
    // The notch routed a zero delta: the accepted offset is where the settle
    // starts, and only the track that settles it moves it.
    assert_eq!(receipt.transitions()[0].current(), block(0));
    assert_eq!(scroll.accepted_offset(), block(0));
    assert_eq!(scroll.displayed_offset(), displayed_before);
    assert_eq!(scroll.world.session.last_scroll_settle_stop(), None);
    assert_eq!(pending_transitions(&scroll), 1);
    let target = scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .transition_target(scroll.owner, scroll.incarnation)
        .expect("the published settle keeps its staged target");
    assert_eq!(target.target_offset(), block(i64::from(LINE_EXTENT_POINTS)));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_notch_over_a_region_declaring_no_line_extent_commits_nothing() {
    let mut scroll = smooth_world(false);

    assert_eq!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Denied(UiHostScrollObservationDenial::SettleUnpublished)
    );
    assert!(
        matches!(
            scroll.world.session.last_scroll_settle_stop(),
            Some(UiScrollSettleStop::Staging { detail }) if detail.contains("OwnerDeclaresNoLineExtent")
        ),
        "the stop names the staging refusal: {:?}",
        scroll.world.session.last_scroll_settle_stop()
    );
    // Staging refused before any successor existed, so no orphaned target
    // waits for a settle that will never run.
    assert_eq!(pending_transitions(&scroll), 0);
    assert!(scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .transition_target(scroll.owner, scroll.incarnation)
        .is_none());
    assert_eq!(scroll.accepted_offset(), block(0));
    let _ = scroll.world.session.shutdown();
}
