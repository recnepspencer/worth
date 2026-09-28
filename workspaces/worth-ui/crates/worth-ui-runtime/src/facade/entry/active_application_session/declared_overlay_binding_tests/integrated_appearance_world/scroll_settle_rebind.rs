//! Facade-level proof that a rebind never lands a settle on another surface
//! generation. A settle owed to the generation a rebind ends is paid while
//! the host still shows that generation's sample, so the content stops where
//! the reader last saw it. A settle that generation cannot pay is released
//! once it has ended and moves nothing. A rebind the host refuses, like a
//! plain deregistration, keeps nothing for a successor that never came.

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

fn rebind_attempt(scroll: &mut ScrollWorld) -> bool {
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
        .is_ok()
}

fn rebind(scroll: &mut ScrollWorld) {
    assert!(rebind_attempt(scroll), "a presented surface rebinds");
}

/// Stage a page on the first component's region without publishing it.
fn stage_page(scroll: &mut ScrollWorld, travel: i64) {
    let (owner, incarnation, target) = (scroll.owner, scroll.incarnation, scroll.target());
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            block(travel),
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .expect("a page with no attempt in flight applies");
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
    stage_page(&mut scroll, 3);

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

/// Register `scroll`'s surface anew and lay it out again once it ended with
/// a page staged and no successor, and hold that neither Scroll nor the
/// mounted geometry kept the page: the new layout stands the content at the
/// offset accepted before the page.
fn assert_no_page_carried(
    scroll: &mut ScrollWorld,
    accepted: crate::runtime::scroll::UiScrollOffset,
    label: &str,
) {
    let surface = scroll.surface();
    assert!(
        !scroll
            .world
            .session
            .mounted
            .has_pending_direct_scroll(surface),
        "{label}: the mounted geometry keeps no page"
    );
    let pending = |scroll: &ScrollWorld| {
        scroll
            .world
            .session
            .scroll
            .as_ref()
            .expect("Scroll stays installed")
            .pending_direct_count()
    };
    assert_eq!(pending(scroll), 0, "{label}: Scroll keeps no page");
    scroll
        .world
        .session
        .register_host_surface(
            surface,
            worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
            crate::mounting::UiSurfaceBindingProfile::new(
                1_000,
                crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                3,
            )
            .expect("a third binding profile"),
        )
        .expect("the ended surface registers anew");
    let (surfaces, instances) = (scroll.world.surfaces, scroll.world.instances);
    super::geometry::scrollable::install_scrollable_primary_with_travel(
        &mut scroll.world.session,
        surfaces,
        instances,
        90,
        60.0,
    );
    assert_eq!(
        pending(scroll),
        0,
        "{label}: no page lands on the new layout"
    );
    assert_eq!(
        scroll.accepted_offset(),
        accepted,
        "{label}: the page never held"
    );
    assert_eq!(
        scroll.mounted_offset(),
        Some(accepted),
        "{label}: the new layout stands where the page was never shown"
    );
}

/// A rebind whose registration the host cannot confirm leaves the surface
/// unbound, as a plain deregistration would: the page staged for the rebound
/// surface is not kept for a successor that never came, so no later
/// registration publishes it.
#[test]
fn a_refused_rebind_keeps_no_page_staged_for_its_successor() {
    let mut scroll = smooth_world(true);
    let accepted = scroll.accepted_offset();
    stage_page(&mut scroll, 3);
    let surface = scroll.surface();
    assert!(scroll
        .world
        .session
        .mounted
        .has_pending_direct_scroll(surface));
    scroll.world.host.return_indeterminate_next_registration();

    assert!(!rebind_attempt(&mut scroll), "the host refuses the rebind");
    scroll
        .world
        .session
        .recover_indeterminate_host_surface(surface)
        .expect("the refused registration recovers");
    assert_no_page_carried(&mut scroll, accepted, "the refused rebind");
    let _ = scroll.world.session.shutdown();
}

/// A surface deregistered with a page staged ends with no successor: the page
/// is kept for no later registration.
#[test]
fn a_deregistered_surface_keeps_no_page_staged_on_it() {
    let mut scroll = smooth_world(true);
    let accepted = scroll.accepted_offset();
    stage_page(&mut scroll, 3);
    let binding = scroll.presentation().binding();
    scroll
        .world
        .session
        .deregister_host_surface(binding)
        .expect("a presented surface deregisters");
    assert_no_page_carried(&mut scroll, accepted, "the deregistration");
    let _ = scroll.world.session.shutdown();
}
