//! A settle tick beside a frame in flight that displaces what the tick would
//! show.
//!
//! Once the frame lands the host draws each command it touches where the
//! frame was prepared. A tick that showed a sample of such a command would
//! stand, with hit testing, where the frame is about to take the command
//! away from, so the tick waits for the frame to land. A group the frame
//! places lands where the frame publishes it, so ticks beside that frame
//! are shown.

use super::super::super::UiScrollSettleDisposition;
use super::geometry::scrollable::install_scrollable_primary_with_travel;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_beside_attempt::rest_at;
use super::scroll_settle_commit::{smooth_world, LINE_EXTENT_POINTS};
use super::scroll_settle_frame::{applied_frames, motion_frame, notch};
use super::scroll_settle_hit_lead::{assert_drawn_where_hit, assert_paint_and_hit_agree};

const NOTCH_TICK: u64 = 5;

/// A settle two frames in, with the scrolled content laid out again over a
/// travel of `travel` points.
fn relaid_out(travel: u16) -> ScrollWorld {
    let mut scroll = smooth_world(true);
    notch(&mut scroll, NOTCH_TICK);
    applied_frames(&mut scroll, NOTCH_TICK, 1, 2);
    let (surfaces, instances) = (scroll.world.surfaces, scroll.world.instances);
    install_scrollable_primary_with_travel(
        &mut scroll.world.session,
        surfaces,
        instances,
        30,
        f32::from(travel),
    );
    scroll
}

fn assert_displaces_samples(scroll: &ScrollWorld, label: &str) {
    assert!(
        !scroll
            .world
            .host
            .held_open_displaced_commands(scroll.surface())
            .is_empty(),
        "{label}: the frame in flight touches the scrolled commands"
    );
}

/// A relayout that keeps the content within its travel changes the commands
/// it draws without placing the group, so the settle waits for it to land
/// and then carries on from where it stood.
#[test]
fn a_tick_waits_for_a_relayout_in_flight_that_displaces_its_samples() {
    let mut scroll = relaid_out(30);
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert_displaces_samples(&scroll, "the relayout");
    let (accepted, mounted) = (scroll.accepted_offset(), scroll.mounted_offset());
    for tick in 4..6 {
        let label = format!("tick {tick} beside the relayout");
        let shown = motion_frame(&mut scroll, NOTCH_TICK + tick, true);
        assert_eq!(shown, Ok(UiScrollSettleDisposition::Idle), "{label}");
        assert_eq!(
            scroll.world.host.pending_presentation_count(),
            1,
            "{label}: no sample is presented"
        );
        scroll.world.host.withdraw_untaken_presentations();
        assert_eq!(
            scroll.accepted_offset(),
            accepted,
            "{label}: nothing staged"
        );
        assert_eq!(scroll.mounted_offset(), mounted, "{label}: nothing shown");
        assert_paint_and_hit_agree(&scroll, &label);
    }
    scroll.complete(pending, NOTCH_TICK + 6);
    assert_paint_and_hit_agree(&scroll, "the relayout landed");
    assert_drawn_where_hit(&scroll, "the relayout landed");
    rest_at(
        &mut scroll,
        7,
        16,
        block(i64::from(LINE_EXTENT_POINTS)),
        "the relayout",
    );
    let _ = scroll.world.session.shutdown();
}

/// A relayout that shrinks the travel below the offset places the group, so
/// ticks beside it are shown and the settle rests at the end of the travel.
#[test]
fn ticks_beside_a_relayout_that_places_the_group_are_shown() {
    let mut scroll = relaid_out(5);
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert_displaces_samples(&scroll, "the relayout");
    for tick in 4..6 {
        let label = format!("tick {tick} beside the relayout");
        let shown = motion_frame(&mut scroll, NOTCH_TICK + tick, true);
        assert!(shown.is_ok(), "{label}: {shown:?}");
        assert_eq!(
            scroll.world.host.pending_presentation_count(),
            0,
            "{label}: the sample is presented"
        );
        assert_paint_and_hit_agree(&scroll, &label);
    }
    scroll.complete(pending, NOTCH_TICK + 6);
    assert_paint_and_hit_agree(&scroll, "the relayout landed");
    assert_drawn_where_hit(&scroll, "the relayout landed");
    rest_at(&mut scroll, 7, 16, block(5), "the relayout");
    let _ = scroll.world.session.shutdown();
}
