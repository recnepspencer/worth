//! Condition postures, admission drift and slot identity for operability
//! read from authored conditions.

use super::*;
use crate::declaration::UiIntentOperabilityDependencyAxis as Axis;
use crate::facade::expression::{
    UiExpressionConditionWithholding as Withholding, UiExpressionCurrentValue,
    UiExpressionDenialReason, UiExpressionOutcome, UiExpressionStaleReason,
    UiExpressionUnavailableReason,
};

/// The withheld condition an axis reports, if it reports one.
fn withheld(
    decision: &UiIntentOperabilityDecision,
    axis: Axis,
) -> Option<UiIntentWithheldCondition> {
    match axis {
        Axis::Mutability => match decision.mutability() {
            UiIntentMutabilityPosture::Withheld(condition) => Some(condition),
            UiIntentMutabilityPosture::Writable | UiIntentMutabilityPosture::Readonly => None,
        },
        Axis::Readiness => match decision.readiness() {
            UiIntentReadinessPosture::Withheld(condition) => Some(condition),
            UiIntentReadinessPosture::Ready | UiIntentReadinessPosture::Pending => None,
        },
        Axis::Policy => match decision.policy() {
            UiIntentPolicyPosture::Withheld(condition) => Some(condition),
            UiIntentPolicyPosture::Admitted | UiIntentPolicyPosture::Denied => None,
        },
    }
}

#[test]
fn each_condition_outcome_keeps_its_own_posture_on_every_axis() {
    for (axis, identity, fact, closed) in [
        (
            Axis::Mutability,
            MUTABLE_WHEN,
            fixture::MUTABLE,
            UiIntentInoperableCause::Readonly,
        ),
        (
            Axis::Readiness,
            READY_WHEN,
            fixture::READY,
            UiIntentInoperableCause::Pending,
        ),
        (
            Axis::Policy,
            POLICY_WHEN,
            fixture::POLICY,
            UiIntentInoperableCause::PolicyDenied,
        ),
    ] {
        let World {
            mut session,
            surface,
            graph,
            ..
        } = world(Policy::Condition);
        let slot = session.expressions.catalog().slot_of(identity).unwrap();
        let closed_class = if closed == UiIntentInoperableCause::Pending {
            Class::Pending
        } else {
            Class::Denied
        };
        let cases = [
            (UiExpressionOutcome::Condition(true), None, Class::Ready),
            (
                UiExpressionOutcome::Condition(false),
                Some(Expected::Closed(closed.clone())),
                closed_class,
            ),
            (
                UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandShapeMismatch {
                    operand: "f".into(),
                }),
                Some(Expected::Withheld(Withholding::Denied)),
                Class::Denied,
            ),
            (
                UiExpressionOutcome::Unavailable(
                    UiExpressionUnavailableReason::ApplicationFactAbsent {
                        operand: "f".into(),
                        fact: fact.into(),
                    },
                ),
                Some(Expected::Withheld(Withholding::Unavailable)),
                Class::Pending,
            ),
            (
                UiExpressionOutcome::Stale {
                    reason: UiExpressionStaleReason::ProjectionRetained {
                        operand: "f".into(),
                        kind:
                            worth_ui_query_binding::UiProjectionRetainedActivityKind::Revalidating,
                    },
                    last_current: Some(UiExpressionCurrentValue::Condition(true)),
                },
                Some(Expected::Withheld(Withholding::Stale)),
                Class::Stale,
            ),
        ];
        for (sequence, (outcome, expected, class)) in (1u64..).step_by(2).zip(cases) {
            session
                .expressions
                .retain_outcome(identity, outcome.clone());
            let (target, decision) = activate(&mut session, surface, sequence);
            let (primary, withholding) = match expected {
                None => (None, None),
                Some(Expected::Closed(cause)) => (Some(cause), None),
                Some(Expected::Withheld(withholding)) => (
                    Some(UiIntentInoperableCause::ConditionWithheld {
                        axis,
                        condition: UiIntentWithheldCondition::new(slot, withholding),
                    }),
                    Some(withholding),
                ),
            };
            assert_eq!(decision.primary_cause(), primary, "{axis:?} {outcome:?}");
            assert_eq!(
                withheld(&decision, axis).map(UiIntentWithheldCondition::withholding),
                withholding,
                "{axis:?} {outcome:?}"
            );
            if withholding.is_some() {
                assert!(
                    !decision.causes().any(|cause| matches!(
                        cause,
                        UiIntentInoperableCause::Readonly
                            | UiIntentInoperableCause::Pending
                            | UiIntentInoperableCause::PolicyDenied
                    )),
                    "a withheld {axis:?} condition is never read as a closed one: {outcome:?}"
                );
            }
            assert_eq!(
                standing(&session, graph, target).class(),
                class,
                "{axis:?} {outcome:?}"
            );
        }
        let _ = session.shutdown();
    }
}

/// The cause a posture case expects besides a withholding.
enum Expected {
    Closed(UiIntentInoperableCause),
    Withheld(Withholding),
}

#[test]
fn a_condition_changed_after_the_proof_stops_admission_under_its_axis() {
    let World {
        mut session,
        surface,
        ..
    } = world(Policy::Condition);
    for (sequence, (fact, expected)) in [
        (
            fixture::MUTABLE,
            UiIntentAdmissionStopReason::OperabilityDependencyChanged,
        ),
        (
            fixture::READY,
            UiIntentAdmissionStopReason::OperabilityDependencyChanged,
        ),
        (fixture::POLICY, UiIntentAdmissionStopReason::PolicyChanged),
    ]
    .into_iter()
    .enumerate()
    {
        let route = activation_route(&mut session, surface, 1 + 2 * sequence as u64);
        let candidate = session.prepare_intent_payload(route).unwrap();
        let proof = session.evaluate_intent_operability(candidate);
        assert!(matches!(proof, UiIntentOperabilityOutcome::Operable(_)));

        set(&mut session, fact, false);

        let UiIntentAdmissionDecision::Stopped(stop) = session.admit_intent(
            UiIntentDefinition::<fixture::Intent>::application_effect(),
            proof,
        ) else {
            panic!("a proof over a changed {fact} condition cannot admit");
        };
        assert_eq!(stop.reason(), &expected, "{fact}");
        set(&mut session, fact, true);
    }
    let _ = session.shutdown();
}

#[test]
fn renumbered_condition_slots_leave_the_intent_contract_equivalent() {
    let role = fixture::role();
    let (plain, _) = fixture::session_with_source(&role, source("f", Policy::Fact));
    let alpha = module("f", Policy::Fact)
        .try_with_condition(
            "test.appearance.alpha",
            [worth_ui_dsl::WorthUiExpressionOperand::new(
                "f",
                worth_ui_dsl::WorthUiExpressionOperandSource::ApplicationBoolean {
                    fact: fixture::READY.to_owned(),
                },
            )],
            "f",
        )
        .unwrap();
    let (renumbered, _) = fixture::session_with_source(
        &role,
        worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([alpha]),
    );
    assert_ne!(
        plain.expressions.catalog().slot_of(MUTABLE_WHEN),
        renumbered.expressions.catalog().slot_of(MUTABLE_WHEN),
        "the leading condition renumbers the consumer's slots"
    );
    assert_eq!(
        plain
            .application
            .prepared_authority()
            .intent_catalog()
            .compare_semantic_contract(
                renumbered.application.prepared_authority().intent_catalog()
            ),
        crate::declaration::UiIntentCatalogSemanticComparison::Equivalent
    );
    let _ = plain.shutdown();
    let _ = renumbered.shutdown();
}
