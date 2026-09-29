use super::session_fixture::{launch, since, COUNT, LABEL, READY};
use crate::facade::expression::{UiExpressionOutcome, UiExpressionWorkCounters};
use crate::facade::intent::{UiIntentApplicationFact, UiIntentBoolean, UiIntentUnsigned64};

const READY_CONDITION: &str =
    "condition ex.ready { operand r application-boolean app.ready; when (r) }";
const NEXT: &str = "derived ex.next { operand c application-unsigned64 app.count; result integer; value (exact_cast<Int64>(c) + 1) }";
const BIG: &str = "condition ex.big { operand c application-unsigned64 app.count; when (exact_cast<Int64>(c) > 100) }";
const BIG_READER: &str = "condition ex.big_reader { operand b condition ex.big; when (b) }";
const LEFT: &str = "condition ex.left { operand r application-boolean app.ready; when (r) }";
const RIGHT: &str = "condition ex.right { operand r application-boolean app.ready; when (r) }";
const JOIN: &str = "condition ex.join { operand l condition ex.left; operand r condition ex.right; when (l && r) }";

fn ready() -> UiIntentApplicationFact<UiIntentBoolean> {
    UiIntentApplicationFact::boolean(READY).unwrap()
}

fn count() -> UiIntentApplicationFact<UiIntentUnsigned64> {
    UiIntentApplicationFact::unsigned64(COUNT).unwrap()
}

#[test]
fn activation_evaluates_each_expression_exactly_once() {
    let session = launch(&format!("{READY_CONDITION}\n{NEXT}"));

    let counters = session.expression_work_counters();
    assert_eq!(counters.evaluations, 2);
    assert_eq!(counters.published_changes, 2);
    assert_eq!(counters.stale_completions, 0);
}

#[test]
fn an_unrelated_fact_update_touches_no_expression_and_no_index_entry() {
    let mut session = launch(READY_CONDITION);
    let before = session.expression_work_counters();

    session.update_intent_unsigned64_fact(&count(), 9).unwrap();

    assert_eq!(
        since(&session, before),
        UiExpressionWorkCounters::default(),
        "nothing reads `app.count`, so nothing is probed, indexed or evaluated"
    );
    let text = UiIntentApplicationFact::text(LABEL, 64).unwrap();
    session.update_intent_text_fact(&text, "busy").unwrap();
    assert_eq!(since(&session, before), UiExpressionWorkCounters::default());
}

#[test]
fn a_fact_update_re_evaluates_only_its_readers() {
    let mut session = launch(&format!("{READY_CONDITION}\n{NEXT}"));
    let before = session.expression_work_counters();

    session.update_intent_boolean_fact(&ready(), true).unwrap();

    let delta = since(&session, before);
    assert_eq!(delta.evaluations, 1, "only `ex.ready` reads `app.ready`");
    assert_eq!(delta.index_hits, 1);
    assert_eq!(delta.published_changes, 1);
}

#[test]
fn an_unchanged_result_publishes_nothing_and_leaves_its_dependents_unevaluated() {
    let mut session = launch(&format!("{BIG}\n{BIG_READER}"));
    let before = session.expression_work_counters();
    let revision = session
        .expression_record("ex.big")
        .unwrap()
        .outcome_revision();

    session.update_intent_unsigned64_fact(&count(), 4).unwrap();

    let delta = since(&session, before);
    assert_eq!(delta.evaluations, 1, "`ex.big` ran; its reader did not");
    assert_eq!(delta.published_changes, 0);
    assert_eq!(delta.suppressed_unchanged, 1);
    let record = session.expression_record("ex.big").unwrap();
    assert_eq!(record.outcome(), &UiExpressionOutcome::Condition(false));
    assert_eq!(record.outcome_revision(), revision, "no new revision");

    session
        .update_intent_unsigned64_fact(&count(), 200)
        .unwrap();
    let delta = since(&session, before);
    assert_eq!(delta.evaluations, 3, "a real change reaches the reader too");
    assert_eq!(delta.published_changes, 2);
    assert_eq!(
        session
            .expression_record("ex.big_reader")
            .unwrap()
            .outcome(),
        &UiExpressionOutcome::Condition(true)
    );
}

#[test]
fn a_diamond_evaluates_its_join_once_per_change() {
    let mut session = launch(&format!("{LEFT}\n{RIGHT}\n{JOIN}"));
    let before = session.expression_work_counters();

    session.update_intent_boolean_fact(&ready(), true).unwrap();

    let delta = since(&session, before);
    assert_eq!(delta.evaluations, 3, "left, right and join, join once");
    assert_eq!(delta.published_changes, 3);
    assert_eq!(
        session.expression_record("ex.join").unwrap().outcome(),
        &UiExpressionOutcome::Condition(true)
    );
}

#[test]
fn a_denied_fact_update_leaves_the_counters_alone() {
    let mut session = launch(READY_CONDITION);
    let before = session.expression_work_counters();
    let unknown = UiIntentApplicationFact::boolean("app.unregistered").unwrap();

    assert!(session.update_intent_boolean_fact(&unknown, true).is_err());

    assert_eq!(since(&session, before), UiExpressionWorkCounters::default());
}
