//! A settle whose samples the host shows beside an attempt in flight, while
//! the attempt's frame places the region.
//!
//! The host keeps showing a command through the last sample it presented
//! until a frame's delta touches that command. So a frame that places a
//! group displaces every sample shown beside it, even a page to where the
//! content already stood, and the content is drawn where hit testing reads
//! it once the frame lands. A settle whose Motion arrives beside a frame
//! that shows a relayout is paid where the arrival was shown, never pulled
//! back to the offset the relayout was lowered from.

use super::super::super::UiScrollSettleDisposition;
use super::geometry::scrollable::install_scrollable_primary_with_travel;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{pending_transitions, smooth_world, LINE_EXTENT_POINTS};
use super::scroll_settle_frame::{applied_frames, motion_frame, notch, owed_frame, quiet_frame};
use super::scroll_settle_hit_lead::{assert_drawn_where_hit, assert_paint_and_hit_agree};
use crate::runtime::scroll::{UiScrollDeltaCause, UiScrollOffset};

const NOTCH_TICK: u64 = 5;

/// Run quiet frames from `first` through `last`, holding the oracles at
/// each, then hold that nothing is left to settle and the content rests at
/// `rests`.
pub(super) fn rest_at(
    scroll: &mut ScrollWorld,
    first: u64,
    last: u64,
    rests: UiScrollOffset,
    label: &str,
) {
    for tick in first..=last {
        let _ = quiet_frame(scroll, NOTCH_TICK + tick);
        assert_paint_and_hit_agree(scroll, &format!("{label}: quiet frame {tick}"));
        assert_drawn_where_hit(scroll, &format!("{label}: quiet frame {tick}"));
    }
    assert_eq!(pending_transitions(scroll), 0, "{label}: nothing strands");
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.accepted_offset(), rests, "{label}: the settle ends");
    assert_eq!(
        scroll.mounted_offset(),
        Some(rests),
        "{label}: where it shows"
    );
}

/// A page to where the content already stands places the group without
/// changing any command it draws. The samples a settle showed beside the
/// frame carrying the page still end once it lands: the frame touches the
/// group's commands, so the host draws them where the page placed them.
#[test]
fn a_page_to_where_the_content_stands_displaces_samples_shown_beside_it() {
    let mut scroll = smooth_world(true);
    notch(&mut scroll, NOTCH_TICK);
    applied_frames(&mut scroll, NOTCH_TICK, 1, 2);
    let accepted = scroll.accepted_offset();
    let (owner, incarnation, target) = (scroll.owner, scroll.incarnation, scroll.target());
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            accepted,
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .expect("a page with no attempt in flight applies");
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert!(
        !scroll
            .world
            .host
            .held_open_displaced_commands(scroll.surface())
            .is_empty(),
        "the page touches the scrolled commands"
    );
    for tick in 4..6 {
        let shown = motion_frame(&mut scroll, NOTCH_TICK + tick, true);
        assert!(
            matches!(
                shown,
                Ok(UiScrollSettleDisposition::DeferredPendingGeometry
                    | UiScrollSettleDisposition::DeferredPresentationInFlight)
            ),
            "the settle shows a sample beside the page: {shown:?}"
        );
        assert_eq!(
            scroll.world.host.pending_presentation_count(),
            0,
            "the sample {tick} beside the page is presented"
        );
        assert_paint_and_hit_agree(&scroll, &format!("sample {tick} beside the page"));
    }
    scroll.complete(pending, NOTCH_TICK + 6);
    assert_paint_and_hit_agree(&scroll, "the page landed");
    assert_drawn_where_hit(&scroll, "the page landed");
    rest_at(&mut scroll, 7, 12, accepted, "the page");
    let _ = scroll.world.session.shutdown();
}

/// A settle whose Motion arrives beside the frame that shows a relayout is
/// paid where the arrival was shown, as far as the new extent reaches.
#[test]
fn an_arrival_shown_beside_a_relayout_in_flight_lands_where_it_was_shown() {
    for travel in [60_u16, 20, 1] {
        let lands = block(i64::from(LINE_EXTENT_POINTS.min(travel)));
        let label = format!("a resize to {travel}");
        let mut scroll = smooth_world(true);
        notch(&mut scroll, NOTCH_TICK);
        applied_frames(&mut scroll, NOTCH_TICK, 1, 6);
        let (surfaces, instances) = (scroll.world.surfaces, scroll.world.instances);
        install_scrollable_primary_with_travel(
            &mut scroll.world.session,
            surfaces,
            instances,
            30,
            f32::from(travel),
        );
        let pending = scroll.hold_presentation_open(NOTCH_TICK + 6);
        let shown = motion_frame(&mut scroll, NOTCH_TICK + 7, true);
        assert!(shown.is_ok(), "{label}: the arrival is shown: {shown:?}");
        assert_paint_and_hit_agree(&scroll, &format!("{label}: the arrival shown"));
        scroll.complete(pending, NOTCH_TICK + 8);
        assert_paint_and_hit_agree(&scroll, &format!("{label}: the relayout landed"));
        assert_drawn_where_hit(&scroll, &format!("{label}: the relayout landed"));
        let _ = owed_frame(&mut scroll);
        // The shell publishes the placement the pulled-back arrival staged.
        scroll.publish_direct(NOTCH_TICK + 9);
        assert_paint_and_hit_agree(&scroll, &format!("{label}: the arrival placed"));
        assert_drawn_where_hit(&scroll, &format!("{label}: the arrival placed"));
        rest_at(&mut scroll, 10, 15, lands, &label);
        let _ = scroll.world.session.shutdown();
    }
}
