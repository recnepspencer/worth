use worth_foundational::expression_api::ExpressionValue;

use super::session_fixture::{launch, COUNT, LABEL, READY};
use crate::facade::expression::{UiExpressionDenialReason, UiExpressionOutcome};
use crate::facade::intent::UiIntentApplicationFact;

const READY_CONDITION: &str =
    "condition ex.ready { operand r application-boolean app.ready; when (r) }";
const ECHO: &str =
    "derived ex.echo { operand l application-text app.label; result text; value (l) }";
const NEXT: &str = "derived ex.next { operand c application-unsigned64 app.count; result integer; value (exact_cast<Int64>(c) + 1) }";
const RATIO: &str = "derived ex.ratio { operand c application-unsigned64 app.count; result integer; value (100 / (exact_cast<Int64>(c) - 3)) }";
const GATE: &str = "condition ex.gate { operand r derived ex.ratio; when (r > 0) }";

fn ready() -> UiIntentApplicationFact<crate::facade::intent::UiIntentBoolean> {
    UiIntentApplicationFact::boolean(READY).unwrap()
}

fn count() -> UiIntentApplicationFact<crate::facade::intent::UiIntentUnsigned64> {
    UiIntentApplicationFact::unsigned64(COUNT).unwrap()
}

fn label() -> UiIntentApplicationFact<crate::facade::intent::UiIntentText> {
    UiIntentApplicationFact::text(LABEL, 64).unwrap()
}

fn outcome(
    session: &crate::facade::WorthUiActiveApplicationSession,
    identity: &str,
) -> UiExpressionOutcome {
    session
        .expression_record(identity)
        .unwrap_or_else(|| panic!("`{identity}` is installed"))
        .outcome()
        .clone()
}

#[test]
fn a_condition_settles_true_and_false_from_its_fact() {
    let mut session = launch(READY_CONDITION);
    assert_eq!(
        outcome(&session, "ex.ready"),
        UiExpressionOutcome::Condition(false)
    );

    session.update_intent_boolean_fact(&ready(), true).unwrap();
    assert_eq!(
        outcome(&session, "ex.ready"),
        UiExpressionOutcome::Condition(true)
    );

    session.update_intent_boolean_fact(&ready(), false).unwrap();
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.outcome(), &UiExpressionOutcome::Condition(false));
    assert_eq!(
        record.outcome_revision(),
        3,
        "each change is its own revision"
    );
}

#[test]
fn derived_text_and_integer_values_follow_their_facts() {
    let mut session = launch(&format!("{ECHO}\n{NEXT}"));
    assert_eq!(
        outcome(&session, "ex.echo"),
        UiExpressionOutcome::Value(ExpressionValue::string("idle"))
    );
    assert_eq!(
        outcome(&session, "ex.next"),
        UiExpressionOutcome::Value(ExpressionValue::integer(4))
    );

    session.update_intent_text_fact(&label(), "busy").unwrap();
    session.update_intent_unsigned64_fact(&count(), 9).unwrap();
    assert_eq!(
        outcome(&session, "ex.echo"),
        UiExpressionOutcome::Value(ExpressionValue::string("busy"))
    );
    assert_eq!(
        outcome(&session, "ex.next"),
        UiExpressionOutcome::Value(ExpressionValue::integer(10))
    );
}

#[test]
fn a_kernel_denial_is_denied_and_never_false() {
    let mut session = launch(&format!("{RATIO}\n{GATE}"));
    assert!(matches!(
        outcome(&session, "ex.ratio"),
        UiExpressionOutcome::Denied(UiExpressionDenialReason::Evaluation(_))
    ));
    assert_eq!(
        outcome(&session, "ex.gate"),
        UiExpressionOutcome::Denied(UiExpressionDenialReason::Upstream {
            operand: "r".into(),
            identity: "ex.ratio".into(),
        }),
        "a denied upstream denies its reader; the reader is not false"
    );

    session.update_intent_unsigned64_fact(&count(), 4).unwrap();
    assert_eq!(
        outcome(&session, "ex.ratio"),
        UiExpressionOutcome::Value(ExpressionValue::integer(100))
    );
    assert_eq!(
        outcome(&session, "ex.gate"),
        UiExpressionOutcome::Condition(true)
    );
}

#[test]
fn a_denied_reader_settles_without_running_the_kernel() {
    let session = launch(&format!("{RATIO}\n{GATE}"));
    let counters = session.expression_work_counters();

    assert_eq!(counters.evaluations, 1, "only the ratio ran the kernel");
    assert_eq!(counters.settled_without_evaluation, 1, "the gate never ran");
    let gate = session.expression_record("ex.gate").unwrap();
    assert!(
        gate.consumed().is_none(),
        "a settled reader consumed nothing"
    );
}
