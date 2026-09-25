//! Facade-level proof that a rebind never lands a settle on another surface
//! generation. A settle owed to the generation a rebind ends is paid while
//! the host still shows that generation's sample, so the content stops where
//! the reader last saw it. A settle that generation cannot pay is released
//! once it has ended and moves nothing.

use super::super::super::scroll_direct_control::scroll_content_motion_target;
use super::super::super::UiScrollSettleDisposition;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::smooth_world;
use super::scroll_settle_frame::{applied_frames, notch, owed_frame, settle_frame};
use crate::runtime::scroll::UiScrollDeltaCause;

const NOTCH_TICK: u64 = 5;

/// Owe a settle through a frame refused mid-presentation, then let the host
/// complete that presentation so the generation that owes it still displays.
fn owe_settle_after_presentation(scroll: &mut ScrollWorld) {
    notch(scroll, NOTCH_TICK);
    applied_frames(scroll, NOTCH_TICK, 1, 2);
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert_eq!(
        settle_frame(scroll, NOTCH_TICK + 3),
        UiScrollSettleDisposition::DeferredPresentationInFlight
    );
    scroll.complete(pending, NOTCH_TICK + 4);
    assert!(scroll.world.session.awaits_scroll_settle_retry());
}

fn rebind(scroll: &mut ScrollWorld) {
    let binding = scroll.presentation().binding();
    scroll
        .world
        .session
        .rebind_host_surface_with_interaction_receipt(
            binding,
            worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
            crate::mounting::UiSurfaceBindingProfile::new(
                1_000,
                crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                2,
            )
            .expect("a second binding profile"),
        )
        .expect("a presented surface rebinds");
}

/// A rebind ends the generation a deferred settle is owed to. The host still
/// shows that generation's sample when the rebind begins, so the content
/// stops there: the settle is paid before the generation ends, never on the
/// new one.
#[test]
fn a_rebind_pays_the_settle_owed_to_the_generation_it_ends() {
    let mut scroll = smooth_world(true);
    owe_settle_after_presentation(&mut scroll);
    let before = scroll.accepted_offset();
    let shown = scroll
        .world
        .session
        .scroll_settlement_reading()
        .displayed_offset(
            scroll_content_motion_target(scroll.owner, scroll.target()),
            scroll.surface(),
        )
        .expect("the generation that owes the settle displays its sample");

    rebind(&mut scroll);
    let paid = scroll.accepted_offset();
    assert!(
        paid.block_subpixels() > before.block_subpixels() && shown.stands_at(paid),
        "the rebind lands the sample the host showed on the generation it ended"
    );
    assert_eq!(scroll.mounted_offset(), Some(paid));
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(
        owed_frame(&mut scroll),
        UiScrollSettleDisposition::Idle,
        "a paid settle is not owed again"
    );
    assert_eq!(scroll.accepted_offset(), paid);
    let _ = scroll.world.session.shutdown();
}

/// A settle the ending generation cannot pay, because a direct placement is
/// staged over its geometry, is never paid on another generation. Once the
/// rebind has ended the generation it was owed to, it is released and moves
/// nothing.
#[test]
fn a_settle_a_rebind_cannot_pay_is_released_not_paid_on_the_new_generation() {
    let mut scroll = smooth_world(true);
    owe_settle_after_presentation(&mut scroll);
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

    rebind(&mut scroll);
    assert!(
        scroll.world.session.awaits_scroll_settle_retry(),
        "a settle deferred behind staged geometry survives the rebind unpaid"
    );
    let accepted = scroll.accepted_offset();
    let displayed = scroll.mounted_offset();

    assert_eq!(
        owed_frame(&mut scroll),
        UiScrollSettleDisposition::Superseded
    );
    assert_eq!(
        scroll.accepted_offset(),
        accepted,
        "a released settle moves nothing"
    );
    assert_eq!(scroll.mounted_offset(), displayed);
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(
        scroll.world.session.last_scroll_settle_disposition(),
        UiScrollSettleDisposition::Superseded
    );
    assert_eq!(
        owed_frame(&mut scroll),
        UiScrollSettleDisposition::Idle,
        "a released settle is not owed again"
    );
    let _ = scroll.world.session.shutdown();
}
