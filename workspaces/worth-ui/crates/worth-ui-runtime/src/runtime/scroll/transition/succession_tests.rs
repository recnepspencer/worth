//! Succession behaviour: what a coarse wheel notch is worth, what a second
//! notch accumulates against, and when a target is extended, re-clamped or
//! retired.

use super::*;

/// The milestone's declared coarse-wheel arithmetic, restated here as plain
/// integers: one notch of three lines against a twenty-point line extent is
/// sixty points, and the Scroll model counts a point as a thousand subpixels.
const LINES_PER_NOTCH: u16 = 3;
const LINE_EXTENT_POINTS: u16 = 20;
const SUBPIXELS_PER_POINT: i64 = 1_000;
const ONE_NOTCH_POINTS: i64 = LINES_PER_NOTCH as i64 * LINE_EXTENT_POINTS as i64;
const ONE_NOTCH_SUBPIXELS: i64 = ONE_NOTCH_POINTS * SUBPIXELS_PER_POINT;
const SETTLE_TICKS: u32 = 8;

fn owner() -> crate::runtime::scroll::UiScrollOwnerIdentity {
    crate::runtime::scroll::UiScrollOwnerIdentity::viewport(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
    )
}

fn incarnation(value: u64) -> crate::runtime::scroll::UiScrollOwnerIncarnation {
    crate::runtime::scroll::UiScrollOwnerIncarnation::new(value).expect("nonzero incarnation")
}

fn basis(accepted_block_subpixels: i64, max_block_subpixels: i64) -> UiScrollTransitionBasis {
    UiScrollTransitionBasis::new(
        crate::runtime::scroll::UiScrollOffset::new(0, accepted_block_subpixels)
            .expect("non-negative offset"),
        crate::runtime::scroll::UiScrollBounds::new(0, max_block_subpixels)
            .expect("non-negative bounds"),
        crate::runtime::scroll::UiScrollAxes::Block,
    )
}

fn notch(block_notches: i64, input_tick: u64) -> UiScrollWheelInput {
    phased_notch(
        block_notches,
        input_tick,
        worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
    )
}

fn phased_notch(
    block_notches: i64,
    input_tick: u64,
    phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
) -> UiScrollWheelInput {
    UiScrollWheelInput::admit(
        UiScrollWheelLineDelta::from_notches(0, block_notches, LINES_PER_NOTCH),
        phase,
        input_tick,
        LINE_EXTENT_POINTS,
        SETTLE_TICKS,
    )
    .expect("a usable line extent and settle horizon")
}

#[test]
fn one_notch_of_three_lines_against_a_twenty_point_line_is_sixty_points() {
    assert_eq!(ONE_NOTCH_POINTS, 60);
    let delta = notch(1, 10).points_subpixels().expect("in range");
    assert_eq!(delta.block_subpixels(), ONE_NOTCH_SUBPIXELS);
    assert_eq!(delta.inline_subpixels(), 0);

    let mut succession = UiScrollTransitionSuccession::new();
    let target = succession
        .accumulate_wheel(owner(), incarnation(1), notch(1, 10), basis(0, 1_000_000))
        .expect("staged");
    assert_eq!(
        target.target_offset().block_subpixels(),
        ONE_NOTCH_SUBPIXELS
    );
}

/// The second notch of a burst adds a full notch to the target the first notch
/// established, not to the accepted sample the content has not reached yet.
#[test]
fn a_second_notch_accumulates_against_the_current_target_not_the_accepted_sample() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    succession
        .accumulate_wheel(owner, incarnation, notch(1, 10), basis(0, 10_000_000))
        .expect("first notch");

    // The accepted sample has barely moved by the time the second notch lands.
    let second = succession
        .accumulate_wheel(
            owner,
            incarnation,
            notch(1, 12),
            basis(ONE_NOTCH_SUBPIXELS / 10, 10_000_000),
        )
        .expect("second notch");

    assert_eq!(
        second.target_offset().block_subpixels(),
        2 * ONE_NOTCH_SUBPIXELS,
        "two notches must be worth two notches of travel"
    );
}

/// Opposite input subtracts from the same authoritative target. Accepted frame
/// progress must not change how much signed wheel travel the reader requested.
#[test]
fn opposite_direction_input_accumulates_against_the_scroll_target() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    succession
        .accumulate_wheel(owner, incarnation, notch(4, 10), basis(0, 10_000_000))
        .expect("forward burst");

    let accepted = ONE_NOTCH_SUBPIXELS;
    let reversed = succession
        .accumulate_wheel(
            owner,
            incarnation,
            notch(-1, 12),
            basis(accepted, 10_000_000),
        )
        .expect("reversal");

    assert_eq!(
        reversed.target_offset().block_subpixels(),
        3 * ONE_NOTCH_SUBPIXELS
    );
}

/// The horizon is measured from the latest input, so a notch arriving mid
/// settle extends the deadline instead of restarting a fixed duration.
#[test]
fn a_mid_settle_notch_extends_the_settle_horizon_from_the_latest_input() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    let first = succession
        .accumulate_wheel(owner, incarnation, notch(1, 10), basis(0, 10_000_000))
        .expect("first notch");
    assert_eq!(first.settle_deadline_tick(), 10 + u64::from(SETTLE_TICKS));

    let second = succession
        .accumulate_wheel(owner, incarnation, notch(1, 14), basis(0, 10_000_000))
        .expect("mid-settle notch");
    assert_eq!(
        second.settle_deadline_tick(),
        14 + u64::from(SETTLE_TICKS),
        "the horizon runs from the latest input, not from the first"
    );
}

/// Elapsed time does not acknowledge a refused endpoint. Later input retains
/// its signed travel and extends the horizon from the new input timestamp.
#[test]
fn an_expired_unaccepted_target_still_accumulates_later_input() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    succession
        .accumulate_wheel(owner, incarnation, notch(1, 10), basis(0, 10_000_000))
        .expect("staged");

    let target = succession
        .accumulate_wheel(owner, incarnation, notch(1, 100), basis(0, 10_000_000))
        .unwrap();
    assert_eq!(
        target.target_offset().block_subpixels(),
        2 * ONE_NOTCH_SUBPIXELS
    );
    assert_eq!(target.settle_deadline_tick(), 100 + u64::from(SETTLE_TICKS));
    assert_eq!(succession.pending_count(), 1);
}

/// Content shrinking under a pending transition pulls the target back inside
/// the reconciled bounds; a target already inside them is retained unchanged.
#[test]
fn reconciling_shrunken_bounds_reclamps_the_pending_target() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    succession
        .accumulate_wheel(owner, incarnation, notch(4, 10), basis(0, 10_000_000))
        .expect("staged");
    let staged = 4 * ONE_NOTCH_SUBPIXELS;

    let roomy = crate::runtime::scroll::UiScrollBounds::new(0, 10_000_000).expect("bounds");
    assert!(matches!(
        succession.reconcile_bounds(owner, incarnation, roomy),
        UiScrollTransitionReclampOutcome::Retained(target)
            if target.target_offset().block_subpixels() == staged
    ));

    let shrunk = crate::runtime::scroll::UiScrollBounds::new(0, 50_000).expect("bounds");
    let UiScrollTransitionReclampOutcome::Reclamped(reclamped) =
        succession.reconcile_bounds(owner, incarnation, shrunk)
    else {
        panic!("a target beyond the reconciled bound must be re-clamped");
    };
    assert_eq!(reclamped.target_offset().block_subpixels(), 50_000);
    assert_eq!(
        reclamped.settle_deadline_tick(),
        10 + u64::from(SETTLE_TICKS),
        "re-clamping must not disturb the settle horizon"
    );
}

/// Content that shrank to fit the space showing it leaves nowhere to settle
/// to. Clamping the target to rest would leave a settle standing with no
/// distance left to travel, and the motion carrying it out would go on
/// translating content with no room to be translated.
#[test]
fn a_collapsed_extent_retires_the_target_rather_than_clamping_it_to_rest() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    succession
        .accumulate_wheel(owner, incarnation, notch(4, 10), basis(0, 10_000_000))
        .expect("staged");
    assert_eq!(succession.pending_count(), 1);

    let collapsed = crate::runtime::scroll::UiScrollBounds::new(0, 0).expect("bounds");
    assert_eq!(
        succession.reconcile_bounds(owner, incarnation, collapsed),
        UiScrollTransitionReclampOutcome::RetiredEmptyExtent
    );
    assert_eq!(succession.pending_count(), 0);
    assert!(succession.target(owner, incarnation).is_none());
    // The retirement is what happened, not a posture the owner now holds. A
    // second reconciliation of the same extent finds nothing to retire.
    assert_eq!(
        succession.reconcile_bounds(owner, incarnation, collapsed),
        UiScrollTransitionReclampOutcome::NoTarget
    );
}

/// A reincarnated owner retires its predecessor's target outright. It is never
/// clamped into the new incarnation, whose content is unrelated.
#[test]
fn an_incarnation_change_retires_the_target_rather_than_clamping_it() {
    let mut succession = UiScrollTransitionSuccession::new();
    let owner = owner();
    succession
        .accumulate_wheel(owner, incarnation(1), notch(2, 10), basis(0, 10_000_000))
        .expect("staged");

    assert!(succession.target(owner, incarnation(2)).is_none());
    assert_eq!(
        succession.reconcile_bounds(
            owner,
            incarnation(2),
            crate::runtime::scroll::UiScrollBounds::new(0, 10_000_000).expect("bounds"),
        ),
        UiScrollTransitionReclampOutcome::RetiredStaleIncarnation
    );
    assert_eq!(succession.pending_count(), 0);
    assert!(succession.target(owner, incarnation(1)).is_none());
}

/// A stale incarnation never accumulates into the surviving target either: the
/// successor stages its own, from its own accepted sample.
#[test]
fn a_stale_incarnation_stages_a_fresh_target_from_its_own_accepted_sample() {
    let mut succession = UiScrollTransitionSuccession::new();
    let owner = owner();
    succession
        .accumulate_wheel(owner, incarnation(1), notch(2, 10), basis(0, 10_000_000))
        .expect("staged");

    let successor = succession
        .accumulate_wheel(owner, incarnation(2), notch(1, 11), basis(0, 10_000_000))
        .expect("staged for the successor");
    assert_eq!(
        successor.target_offset().block_subpixels(),
        ONE_NOTCH_SUBPIXELS
    );
    assert_eq!(successor.incarnation(), incarnation(2));
    assert_eq!(succession.pending_count(), 1);
}

/// A burst of many events holds exactly one target per owner: storage is bound
/// by live owners, never by the number of events delivered.
#[test]
fn storage_is_bounded_by_owners_not_by_event_count() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    succession
        .accumulate_wheel(
            owner,
            incarnation,
            phased_notch(
                1,
                1,
                worth_ui_host_contract::UiHostScrollDeltaPhase::Started,
            ),
            basis(0, 100_000_000),
        )
        .expect("burst opens");
    for tick in 2..200 {
        succession
            .accumulate_wheel(owner, incarnation, notch(1, tick), basis(0, 100_000_000))
            .expect("burst continues");
        assert_eq!(succession.pending_count(), 1);
    }
    assert!(succession
        .accumulation_window(owner)
        .expect("window")
        .is_open());

    succession
        .accumulate_wheel(
            owner,
            incarnation,
            phased_notch(
                1,
                200,
                worth_ui_host_contract::UiHostScrollDeltaPhase::Ended,
            ),
            basis(0, 100_000_000),
        )
        .expect("burst closes");
    let window = succession.accumulation_window(owner).expect("window");
    assert!(!window.is_open());
    assert_eq!(window.latest_input_tick(), 200);
    assert_eq!(succession.pending_count(), 1);

    assert!(succession.retire(owner));
    assert!(!succession.retire(owner));
    assert_eq!(succession.pending_count(), 0);
}

/// A staged target names the owner, the incarnation and the horizon it is
/// bound to, so a consumer never has to infer any of the three.
#[test]
fn a_staged_target_names_its_owner_incarnation_and_remaining_horizon() {
    let mut succession = UiScrollTransitionSuccession::new();
    let (owner, incarnation) = (owner(), incarnation(1));
    let target = succession
        .accumulate_wheel(owner, incarnation, notch(1, 10), basis(0, 10_000_000))
        .expect("staged");

    assert_eq!(target.owner(), owner);
    assert_eq!(target.incarnation(), incarnation);
    assert_eq!(target.horizon().latest_input_tick(), 10);
    assert_eq!(
        target.horizon().remaining_ticks(10),
        u64::from(SETTLE_TICKS)
    );
    assert_eq!(
        target.horizon().remaining_ticks(14),
        u64::from(SETTLE_TICKS) - 4
    );
    assert_eq!(
        target
            .horizon()
            .remaining_ticks(10 + u64::from(SETTLE_TICKS)),
        0
    );
}
