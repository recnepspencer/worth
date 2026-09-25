//! The binding between authoritative owner records and pending transition
//! targets: staging reads the owner's accepted geometry, and every path that
//! reconciles bounds drags the target with it or retires it.

use super::transition::{UiScrollTransitionDenial, UiScrollWheelInput, UiScrollWheelLineDelta};
use super::*;

const LINE_EXTENT_POINTS: u16 = 20;
const SETTLE_TICKS: u32 = 8;
/// Three lines per notch, twenty points per line, a thousand subpixels per point.
const ONE_NOTCH_SUBPIXELS: i64 = 3 * 20 * 1_000;

fn surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface")
}

fn incarnation(value: u64) -> UiScrollOwnerIncarnation {
    UiScrollOwnerIncarnation::new(value).expect("nonzero incarnation")
}

fn bounds(block: i64) -> UiScrollBounds {
    UiScrollBounds::new(0, block).expect("non-negative bounds")
}

fn registration(
    owner: UiScrollOwnerIdentity,
    incarnation: UiScrollOwnerIncarnation,
    block_bound: i64,
    block_offset: i64,
) -> UiScrollOwnerRegistration {
    UiScrollOwnerRegistration::new(
        owner,
        incarnation,
        UiScrollAxes::Block,
        bounds(block_bound),
        UiScrollOffset::new(0, block_offset).expect("non-negative offset"),
    )
}

fn notch(block_notches: i64, input_tick: u64) -> UiScrollWheelInput {
    UiScrollWheelInput::admit(
        UiScrollWheelLineDelta::from_notches(0, block_notches, 3),
        worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
        input_tick,
        LINE_EXTENT_POINTS,
        SETTLE_TICKS,
    )
    .expect("a usable line extent and settle horizon")
}

fn host_cause() -> UiScrollDeltaCause {
    UiScrollDeltaCause::Host {
        source: worth_ui_host_contract::UiHostScrollDeltaSource::PointerWheel,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision::Pixel,
    }
}

#[test]
fn staging_reads_the_owners_accepted_offset_bounds_and_axis_policy() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 5_000))
        .expect("registration");

    let target = state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(1, 10))
        .expect("staged");

    assert_eq!(
        target.target_offset().block_subpixels(),
        5_000 + ONE_NOTCH_SUBPIXELS,
        "the target leads the accepted offset, not the origin"
    );
    assert_eq!(state.transition_target(owner, incarnation(1)), Some(target));
    assert_eq!(state.pending_transition_count(), 1);
}

/// A block-only owner cannot be given inline travel by the transition path any
/// more than by the routed path: the owner's axis policy governs both.
#[test]
fn staging_respects_the_owners_axis_policy_and_bound() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 10_000, 0))
        .expect("registration");

    let input = UiScrollWheelInput::admit(
        UiScrollWheelLineDelta::from_notches(4, 4, 3),
        worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
        10,
        LINE_EXTENT_POINTS,
        SETTLE_TICKS,
    )
    .expect("admitted");
    let target = state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), input)
        .expect("staged");

    assert_eq!(target.target_offset().inline_subpixels(), 0);
    assert_eq!(target.target_offset().block_subpixels(), 10_000);
}

#[test]
fn staging_denies_an_unknown_owner_and_a_stale_incarnation() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    assert_eq!(
        state.stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(1, 10)),
        Err(UiScrollTransitionDenial::UnknownOwner)
    );

    state
        .register(registration(owner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    assert_eq!(
        state.stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(2)), notch(1, 10)),
        Err(UiScrollTransitionDenial::StaleIncarnation)
    );
    assert_eq!(state.pending_transition_count(), 0);
}

/// Rebinding the same incarnation onto shrunken content re-clamps the pending
/// target in the same reconciliation that moved the accepted offset.
#[test]
fn a_rebind_that_shrinks_bounds_reclamps_the_pending_target() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    let staged = state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(4, 10))
        .expect("staged");
    assert_eq!(
        staged.target_offset().block_subpixels(),
        4 * ONE_NOTCH_SUBPIXELS
    );

    state
        .reconcile_rebind(UiScrollRebindRequest::new(
            registration(owner, incarnation(1), 30_000, 0),
            None,
            UiScrollAnchorPolicy::Clamp,
        ))
        .expect("rebind");

    assert_eq!(
        state
            .transition_target(owner, incarnation(1))
            .expect("still pending")
            .target_offset()
            .block_subpixels(),
        30_000
    );
}

/// A reincarnated owner retires its predecessor's target on the rebind that
/// installs it. It is never clamped into the successor's content.
#[test]
fn a_rebind_into_a_new_incarnation_retires_the_predecessors_target() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(2, 10))
        .expect("staged");

    state
        .reconcile_rebind(UiScrollRebindRequest::new(
            registration(owner, incarnation(2), 1_000_000, 0),
            None,
            UiScrollAnchorPolicy::Clamp,
        ))
        .expect("rebind");

    assert_eq!(state.pending_transition_count(), 0);
    assert!(state.transition_target(owner, incarnation(2)).is_none());
}

/// Routing with allocation-derived bounds reconciles the pending target too, so
/// a target cannot survive a route that narrowed the content under it.
#[test]
fn routing_with_reconciled_bounds_reclamps_the_pending_target() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(4, 10))
        .expect("staged");

    state
        .route_with_reconciled_bounds(
            UiScrollDeltaRequest::new(
                vec![UiScrollChainEntry::new(owner, incarnation(1))],
                UiScrollDelta::new(0, 1_000),
                host_cause(),
            )
            .expect("request"),
            &[bounds(20_000)],
        )
        .expect("route");

    assert_eq!(
        state
            .transition_target(owner, incarnation(1))
            .expect("still pending")
            .target_offset()
            .block_subpixels(),
        20_000
    );
}

/// Reconciled bounds reach the owners a route never visited, not only the ones
/// it did.
///
/// A route stops at the first owner that leaves nothing over, so the zero delta
/// a declared smooth wheel routes visits exactly one owner however deep the
/// chain is. The ancestor behind it is the owner that notch will actually
/// settle, and the bounds arriving with the route are the live geometry it has
/// to settle against. Writing bounds only as far as the route travelled would
/// leave that ancestor measuring itself against whatever geometry was current
/// the last time a delta had something left for it -- under a smooth wheel,
/// never -- and it would stage a target past content that is no longer there.
///
/// The outer owner is asked afterwards by staging against it: a target clamped
/// to the reconciled extent proves the bounds landed, and one clamped to the
/// registered extent proves they stopped short.
#[test]
fn reconciled_bounds_reach_a_chain_owner_the_route_never_visited() {
    let surface = surface();
    let inner =
        UiScrollOwnerIdentity::region(surface, crate::graph::UiGraphNodeIdentity::new(11), 1);
    let outer = UiScrollOwnerIdentity::viewport(surface);
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(inner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    state
        .register(registration(outer, incarnation(2), 1_000_000, 0))
        .expect("registration");

    let receipt = state
        .route_with_reconciled_bounds(
            UiScrollDeltaRequest::new(
                vec![
                    UiScrollChainEntry::new(inner, incarnation(1)),
                    UiScrollChainEntry::new(outer, incarnation(2)),
                ],
                UiScrollDelta::new(0, 0),
                host_cause(),
            )
            .expect("request"),
            &[bounds(1_000_000), bounds(20_000)],
        )
        .expect("route");
    assert_eq!(
        receipt.owners_visited(),
        1,
        "a zero delta leaves nothing over, so the outer owner is never visited"
    );

    let target = state
        .stage_wheel_transition(UiScrollChainEntry::new(outer, incarnation(2)), notch(2, 10))
        .expect("staged");
    assert_eq!(
        target.target_offset().block_subpixels(),
        20_000,
        "the unvisited owner settles against the extent the route reconciled,          not the one it was registered with"
    );
}

/// Only accepted arrival or explicit cancellation retires pending intent;
/// shutdown releases whatever remains so storage never outlives the session.
#[test]
fn accepted_arrival_cancellation_and_shutdown_release_pending_targets() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(1, 10))
        .expect("staged");

    assert_eq!(state.retire_reached_transitions(), 0);
    state
        .settle_accepted_sample(
            UiScrollChainEntry::new(owner, incarnation(1)),
            crate::mounting::presentation::displayed_scroll_offset_for_test(
                UiScrollOffset::new(0, ONE_NOTCH_SUBPIXELS).unwrap(),
            ),
            bounds(1_000_000),
        )
        .unwrap();
    assert_eq!(state.retire_reached_transitions(), 1);
    assert_eq!(state.pending_transition_count(), 0);

    state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(1, 30))
        .expect("staged again");
    assert!(state.retire_transition(owner));
    assert!(!state.retire_transition(owner));

    state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(1, 40))
        .expect("staged once more");
    state.shutdown();
    assert_eq!(state.pending_transition_count(), 0);
}

/// A reveal places the offset outright, so a settle still walking that owner
/// toward a wheel target is an intention the reveal has overruled. Left
/// standing, the track behind it would drag the content straight back off
/// whatever the reveal just brought into view.
#[test]
fn a_reveal_retires_the_settle_it_overrode() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 0))
        .expect("registration");
    state
        .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation(1)), notch(1, 10))
        .expect("staged");
    assert_eq!(state.pending_transition_count(), 1);

    state
        .reveal(
            UiScrollProgrammaticRevealRequest::new(
                vec![UiScrollChainEntry::new(owner, incarnation(1))],
                UiScrollRevealTarget::new(
                    UiScrollRevealInterval::new(0, 10).expect("an ordered interval"),
                    UiScrollRevealInterval::new(400_000, 460_000).expect("an ordered interval"),
                ),
                UiScrollViewportExtent::new(100_000, 100_000).expect("a positive extent"),
                crate::declaration::UiScrollRevealAlignment::End,
            )
            .expect("a revealable chain"),
        )
        .expect("the reveal routes");

    assert_eq!(
        state
            .offset(owner, incarnation(1))
            .expect("an owner offset"),
        UiScrollOffset::new(0, 360_000).expect("a non-negative offset"),
        "the reveal placed the offset it asked for"
    );
    assert_eq!(
        state.pending_transition_count(),
        0,
        "nothing may go on travelling toward a target the reveal overrode"
    );
}

/// Revealing something that is already at the top asks, arithmetically, to
/// scroll behind the beginning of the content. There is no such place, so the
/// request lands at rest rather than being asserted away.
#[test]
fn a_reveal_behind_rest_lands_at_rest() {
    let owner = UiScrollOwnerIdentity::viewport(surface());
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(registration(owner, incarnation(1), 1_000_000, 40_000))
        .expect("registration");

    state
        .reveal(
            UiScrollProgrammaticRevealRequest::new(
                vec![UiScrollChainEntry::new(owner, incarnation(1))],
                UiScrollRevealTarget::new(
                    UiScrollRevealInterval::new(0, 10).expect("an ordered interval"),
                    UiScrollRevealInterval::new(0, 5_000).expect("an ordered interval"),
                ),
                UiScrollViewportExtent::new(100_000, 100_000).expect("a positive extent"),
                crate::declaration::UiScrollRevealAlignment::End,
            )
            .expect("a revealable chain"),
        )
        .expect("the reveal routes");

    assert_eq!(
        state
            .offset(owner, incarnation(1))
            .expect("an owner offset"),
        UiScrollOffset::origin(),
        "the content is as far back as content goes"
    );
}
