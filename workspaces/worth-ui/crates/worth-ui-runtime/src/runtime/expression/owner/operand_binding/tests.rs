use worth_foundational::expression_api::ExpressionValue;
use worth_ui_query_binding::{
    UiProjectionFactStopKind, UiProjectionInputPosture, UiProjectionInputTransitionStopKind,
    UiProjectionRetainedActivityKind, UiProjectionUnavailableKind,
};

use super::{posture_observation, severity, withhold, Observation};
use crate::runtime::expression::{
    UiExpressionDenialReason, UiExpressionOutcome, UiExpressionStaleReason, UiExpressionStopKind,
    UiExpressionUnavailableReason,
};

fn stale(operand: &str) -> UiExpressionOutcome {
    UiExpressionOutcome::Stale {
        reason: UiExpressionStaleReason::ProjectionRetained {
            operand: operand.into(),
            kind: UiProjectionRetainedActivityKind::Idle,
        },
        last_current: None,
    }
}

fn unavailable(operand: &str) -> UiExpressionOutcome {
    UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
        operand: operand.into(),
    })
}

fn denied(operand: &str) -> UiExpressionOutcome {
    UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandWrongWorld {
        operand: operand.into(),
    })
}

fn settled_by(outcomes: impl IntoIterator<Item = UiExpressionOutcome>) -> UiExpressionOutcome {
    let mut settled = None;
    for outcome in outcomes {
        withhold(&mut settled, outcome);
    }
    settled.expect("at least one operand withheld")
}

fn non_current(observation: Observation) -> UiExpressionOutcome {
    match observation {
        Observation::NonCurrent(outcome) => outcome,
        Observation::Bound(value) => panic!("a posture bound the value {value:?}"),
    }
}

fn observe(posture: UiProjectionInputPosture) -> UiExpressionOutcome {
    non_current(posture_observation("p", posture, || {
        panic!("a non-current posture never reads its value")
    }))
}

#[test]
fn a_wrong_world_projection_is_denied_as_wrong_world_never_false() {
    assert_eq!(
        observe(UiProjectionInputPosture::Stopped(
            UiProjectionFactStopKind::WrongWorld
        )),
        denied("p")
    );
}

#[test]
fn every_other_stop_is_a_denial_that_names_its_kind() {
    assert_eq!(
        observe(UiProjectionInputPosture::Stopped(
            UiProjectionFactStopKind::BasisMismatch
        )),
        UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandStopped {
            operand: "p".into(),
            kind: UiExpressionStopKind::Projection(UiProjectionFactStopKind::BasisMismatch),
        })
    );
    assert_eq!(
        observe(UiProjectionInputPosture::TransitionStopped(
            UiProjectionInputTransitionStopKind::MalformedPatch
        )),
        UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandStopped {
            operand: "p".into(),
            kind: UiExpressionStopKind::Transition(
                UiProjectionInputTransitionStopKind::MalformedPatch
            ),
        })
    );
}

#[test]
fn a_retained_stale_projection_is_stale_and_carries_no_last_value_yet() {
    assert_eq!(
        observe(UiProjectionInputPosture::RetainedStale(
            UiProjectionRetainedActivityKind::Revalidating
        )),
        UiExpressionOutcome::Stale {
            reason: UiExpressionStaleReason::ProjectionRetained {
                operand: "p".into(),
                kind: UiProjectionRetainedActivityKind::Revalidating,
            },
            last_current: None,
        }
    );
}

#[test]
fn an_unavailable_projection_is_unavailable_with_its_kind() {
    assert_eq!(
        observe(UiProjectionInputPosture::Unavailable(
            UiProjectionUnavailableKind::Pending
        )),
        UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Projection {
            operand: "p".into(),
            kind: UiProjectionUnavailableKind::Pending,
        })
    );
}

#[test]
fn only_a_current_projection_reads_its_value() {
    let observation = posture_observation("p", UiProjectionInputPosture::Current, || {
        Observation::Bound(ExpressionValue::string("ONLINE"))
    });

    assert!(matches!(
        observation,
        Observation::Bound(value) if value == ExpressionValue::string("ONLINE")
    ));
}

#[test]
fn severity_ranks_denied_over_unavailable_over_stale() {
    assert!(severity(&denied("a")) > severity(&unavailable("a")));
    assert!(severity(&unavailable("a")) > severity(&stale("a")));
    assert!(severity(&stale("a")) > severity(&UiExpressionOutcome::Condition(true)));
}

#[test]
fn a_denied_operand_outranks_an_earlier_stale_or_unavailable_one() {
    assert_eq!(settled_by([stale("a"), denied("z")]), denied("z"));
    assert_eq!(settled_by([unavailable("a"), denied("z")]), denied("z"));
    assert_eq!(
        settled_by([stale("a"), unavailable("m"), denied("z")]),
        denied("z")
    );
}

#[test]
fn an_unavailable_operand_outranks_an_earlier_stale_one() {
    assert_eq!(settled_by([stale("a"), unavailable("z")]), unavailable("z"));
}

#[test]
fn a_later_less_severe_operand_never_replaces_a_more_severe_one() {
    assert_eq!(settled_by([denied("a"), unavailable("z")]), denied("a"));
    assert_eq!(settled_by([unavailable("a"), stale("z")]), unavailable("a"));
}

#[test]
fn name_order_breaks_a_tie() {
    assert_eq!(settled_by([stale("a"), stale("z")]), stale("a"));
    assert_eq!(settled_by([denied("a"), denied("z")]), denied("a"));
}
