//! A pointer reaches scrolled content where the host shows it while a settle
//! is owed, and where the committed pose puts it once nothing is owed.
//!
//! A settle deferred behind an open attempt leaves the host showing the
//! accepted sample while committed geometry stays behind it, so the hit rows
//! lead to the sample. Whatever discharges the settle, the rows then stand
//! where committed geometry puts them: no lead outlives the settle it served.
//! The paint/hit oracle holds at every step. Mounted geometry staged past the frame the host
//! shows, by a page or a resize no witness has shown, moves no lead: the rows
//! lead from where they committed.

use super::super::super::UiScrollSettleDisposition;
use super::geometry::scrollable::install_scrollable_primary_with_travel;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, smooth_world, ONE_NOTCH};
use super::scroll_settle_frame::{applied_frames, motion_frame, notch, owed_frame, settle_frame};
use crate::runtime::scroll::UiScrollDeltaCause;

const NOTCH_TICK: u64 = 5;

pub(super) fn assert_paint_and_hit_agree(scroll: &ScrollWorld, label: &str) {
    let parted = scroll
        .world
        .session
        .mounted
        .parted_paint_and_hit(scroll.surface());
    assert!(
        parted.is_empty(),
        "{label}: paint and hit testing part: {parted:?}"
    );
}

/// The host draws the nested row where interaction reads it, to the device
/// pixel it snaps a sample to.
pub(super) fn assert_drawn_where_hit(scroll: &ScrollWorld, label: &str) {
    let nested = scroll.world.instances[2];
    let hit = nested_hit_y(scroll);
    for (_, drawn) in scroll
        .world
        .host
        .accepted_text_drawn_bounds(scroll.surface())
        .into_iter()
        .filter(|(identity, _)| identity.mounted_instance() == nested)
    {
        assert!(
            hit.is_some_and(|hit| (drawn.y() - hit).abs() <= 0.5),
            "{label}: the host draws the nested row at {}, hit testing reads it at {hit:?}",
            drawn.y()
        );
    }
}

/// Where interaction reads the nested row, if it reads it at all.
pub(super) fn nested_hit_y(scroll: &ScrollWorld) -> Option<f32> {
    let nested = scroll.world.instances[2];
    scroll
        .world
        .session
        .mounted
        .interaction_hit_test_basis(scroll.presentation())
        .expect("the displayed frame is readable")
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == nested)
        .map(|row| row.bounds().platform_box().y())
}

/// Defer one settle frame behind an open attempt, and return the hit row's
/// place before the frame.
fn owe_behind_open_attempt(scroll: &mut ScrollWorld) -> Option<f32> {
    notch(scroll, NOTCH_TICK);
    applied_frames(scroll, NOTCH_TICK, 1, 2);
    let settled = nested_hit_y(scroll);
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert_eq!(
        settle_frame(scroll, NOTCH_TICK + 3),
        UiScrollSettleDisposition::DeferredPresentationInFlight
    );
    assert!(scroll.world.session.awaits_scroll_settle_retry());
    assert_paint_and_hit_agree(scroll, "owed behind the open attempt");
    assert!(
        nested_hit_y(scroll) != settled,
        "the rows lead to the sample the host shows past the settled pose"
    );
    scroll.complete(pending, NOTCH_TICK + 4);
    assert_paint_and_hit_agree(scroll, "the open attempt completed");
    settled
}

#[test]
fn an_owed_settle_paid_by_its_commit_leaves_the_rows_where_it_committed() {
    let mut scroll = smooth_world(true);
    owe_behind_open_attempt(&mut scroll);
    assert_eq!(owed_frame(&mut scroll), UiScrollSettleDisposition::Applied);
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_paint_and_hit_agree(&scroll, "the owed settle paid");
    let _ = scroll.world.session.shutdown();
}

#[test]
fn an_owed_settle_a_page_replaces_leaves_no_lead_behind() {
    let mut scroll = smooth_world(true);
    owe_behind_open_attempt(&mut scroll);
    let (owner, incarnation, target) = (scroll.owner, scroll.incarnation, scroll.target());
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            block(3),
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .expect("a page with no attempt in flight applies");
    scroll.publish_direct(NOTCH_TICK + 6);
    let discharged = owed_frame(&mut scroll);
    assert!(
        !matches!(
            discharged,
            UiScrollSettleDisposition::DeferredPresentationInFlight
                | UiScrollSettleDisposition::DeferredPendingGeometry
        ),
        "nothing holds the settle open any more: {discharged:?}"
    );
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.mounted_offset(), Some(block(3)));
    assert_paint_and_hit_agree(&scroll, "the page replaced the owed settle");
    // A later wheel starts from the page, not from a lead the settle left.
    let _ = scroll.wheel(ONE_NOTCH, one_notch_up(), NOTCH_TICK + 7);
    assert_paint_and_hit_agree(&scroll, "a notch after the page");
    let _ = scroll.world.session.shutdown();
}

/// Show the owed sample's successor over geometry staged past the frame the
/// host shows, then publish what was staged.
fn show_a_sample_over_staged_geometry(scroll: &mut ScrollWorld, label: &str) {
    let shown = motion_frame(scroll, NOTCH_TICK + 5, true);
    assert!(
        matches!(
            shown,
            Ok(UiScrollSettleDisposition::DeferredPendingGeometry)
        ),
        "{label}: the staged geometry holds the settle owed: {shown:?}"
    );
    assert_paint_and_hit_agree(scroll, &format!("{label}: a sample over it"));
    scroll.publish_direct(NOTCH_TICK + 6);
    assert_paint_and_hit_agree(scroll, &format!("{label}: published, its settle owed"));
    assert_drawn_where_hit(scroll, &format!("{label}: published, its settle owed"));
    let _ = owed_frame(scroll);
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_paint_and_hit_agree(scroll, &format!("{label}: published"));
}

#[test]
fn a_sample_shown_over_a_staged_page_leads_from_where_the_rows_committed() {
    for points in [0, 1, 3] {
        let mut scroll = smooth_world(true);
        owe_behind_open_attempt(&mut scroll);
        let (owner, incarnation, target) = (scroll.owner, scroll.incarnation, scroll.target());
        scroll
            .world
            .session
            .place_scroll_chrome_offset(
                owner,
                incarnation,
                target,
                0,
                block(points),
                UiScrollDeltaCause::ChromeTrackPage,
            )
            .expect("a page with no attempt in flight applies");
        assert_eq!(scroll.mounted_offset(), Some(block(points)));
        show_a_sample_over_staged_geometry(&mut scroll, &format!("a page to {points}"));
        let _ = scroll.world.session.shutdown();
    }
}

#[test]
fn a_sample_shown_over_a_staged_resize_leads_from_where_the_rows_committed() {
    // The shorter travel clamps the staged pose below the shown sample.
    for travel in [20.0, 1.0] {
        let mut scroll = smooth_world(true);
        owe_behind_open_attempt(&mut scroll);
        let (surfaces, instances) = (scroll.world.surfaces, scroll.world.instances);
        install_scrollable_primary_with_travel(
            &mut scroll.world.session,
            surfaces,
            instances,
            30,
            travel,
        );
        assert!(scroll
            .world
            .session
            .scroll
            .as_ref()
            .is_some_and(|installed| installed.has_unpresented_layout(scroll.surface())));
        show_a_sample_over_staged_geometry(&mut scroll, &format!("a resize to {travel}"));
        let _ = scroll.world.session.shutdown();
    }
}

/// A layout that lands after the settle's Motion has arrived places the
/// group where the arrival was shown, as far as the new extent reaches: the
/// settle has no track left to rebase, so the staged layout follows the
/// arrival, the frame that shows it publishes the group there, and nothing
/// is left waiting on it. The content never steps back to the offset the
/// layout was staged from.
#[test]
fn an_arrival_shown_over_a_staged_resize_lands_where_it_was_shown() {
    use super::scroll_settle_commit::{pending_transitions, LINE_EXTENT_POINTS};
    use super::scroll_settle_frame::quiet_frame;
    // The shortest travel clamps the arrival to the new extent.
    for travel in [60_u16, 20, 1] {
        let lands = i64::from(LINE_EXTENT_POINTS.min(travel));
        let label = format!("a resize to {travel}");
        let mut scroll = smooth_world(true);
        notch(&mut scroll, NOTCH_TICK);
        applied_frames(&mut scroll, NOTCH_TICK, 1, 6);
        assert!(
            pending_transitions(&scroll) > 0,
            "the notch is still settling"
        );
        let (surfaces, instances) = (scroll.world.surfaces, scroll.world.instances);
        install_scrollable_primary_with_travel(
            &mut scroll.world.session,
            surfaces,
            instances,
            30,
            f32::from(travel),
        );
        // The arrival is shown over the staged resize, so its settle is owed.
        assert!(matches!(
            motion_frame(&mut scroll, NOTCH_TICK + 7, true),
            Ok(UiScrollSettleDisposition::DeferredPendingGeometry)
        ));
        assert_paint_and_hit_agree(&scroll, &format!("{label}: the arrival shown"));
        scroll.publish_direct(NOTCH_TICK + 8);
        let placed = block(lands);
        assert_eq!(
            scroll.mounted_offset(),
            Some(placed),
            "{label}: the layout places the group where the arrival was shown"
        );
        assert_paint_and_hit_agree(&scroll, &format!("{label}: published"));
        assert_drawn_where_hit(&scroll, &format!("{label}: published"));
        let _ = owed_frame(&mut scroll);
        for tick in 9..14 {
            let _ = quiet_frame(&mut scroll, NOTCH_TICK + tick);
            assert_paint_and_hit_agree(&scroll, &format!("{label}: quiet frame {tick}"));
            assert_drawn_where_hit(&scroll, &format!("{label}: quiet frame {tick}"));
        }
        assert_eq!(pending_transitions(&scroll), 0, "{label}: nothing strands");
        assert!(!scroll.world.session.awaits_scroll_settle_retry());
        assert_eq!(scroll.accepted_offset(), placed, "{label}: the settle ends");
        assert_eq!(
            scroll.mounted_offset(),
            Some(placed),
            "{label}: where it shows"
        );
        let _ = scroll.world.session.shutdown();
    }
}

/// A frame bound while an attempt is in flight stands its groups where the
/// host shows them once the attempt lands: samples displayed beside the
/// attempt reach the groups its frame bound, so a frame bound after it moves
/// displayed commands from where they stand, not from the published offset.
#[test]
fn samples_shown_beside_an_attempt_reach_the_groups_its_frame_bound() {
    let mut scroll = smooth_world(true);
    let pending = scroll.hold_presentation_open(NOTCH_TICK - 2);
    notch(&mut scroll, NOTCH_TICK);
    for tick in 1..4 {
        assert_eq!(
            motion_frame(&mut scroll, NOTCH_TICK + tick, true),
            Ok(UiScrollSettleDisposition::DeferredPresentationInFlight)
        );
        assert_paint_and_hit_agree(&scroll, &format!("sample {tick} beside the attempt"));
    }
    scroll.complete(pending, NOTCH_TICK + 4);
    assert_paint_and_hit_agree(&scroll, "the attempt landed");
    // A later frame binds the groups again, reading each command's
    // displayed base against the standing the landed frame bound.
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 5);
    scroll.complete(pending, NOTCH_TICK + 6);
    assert_eq!(owed_frame(&mut scroll), UiScrollSettleDisposition::Applied);
    for tick in 7..14 {
        let _ = motion_frame(&mut scroll, NOTCH_TICK + tick, true);
        assert_paint_and_hit_agree(&scroll, &format!("frame {tick} after rebinding"));
        assert_drawn_where_hit(&scroll, &format!("frame {tick} after rebinding"));
    }
    let _ = scroll.world.session.shutdown();
}
