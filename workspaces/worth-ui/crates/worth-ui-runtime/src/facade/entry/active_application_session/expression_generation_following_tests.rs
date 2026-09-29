use std::sync::Arc;

use super::expression_session_fixture::{complete, evaluate, evidence_only_rebind, session_inputs};
use super::succession_characterization::{
    assert_owners_follow, assert_pointer_is_fresh, assert_standing_is_fresh, SuccessionWork,
};
use crate::facade::expression::{
    UiExpressionCurrentValue, UiExpressionOutcome, UiExpressionStaleReason,
};
use crate::facade::intent::UiIntentApplicationFact;
use crate::runtime::expression::{UiExpressionCompletionReceipt, UiExpressionRuntimeState};
use crate::runtime::tests::expression::cutover_fixture::cut_over;
use crate::runtime::tests::expression::session_fixture::{launch, since};

const READY_CONDITION: &str =
    "condition ex.ready { operand r application-boolean app.ready; when (r) }";

const COMMENTED_CONDITION: &str =
    "condition ex.ready { operand r application-boolean app.ready; // trivia\n when (r) }";

const NEGATED_CONDITION: &str =
    "condition ex.ready { operand r application-boolean app.ready; when (!r) }";

const READER: &str = "condition ex.reader { operand up condition ex.ready; when (up) }";

fn retained_stale() -> UiExpressionOutcome {
    UiExpressionOutcome::Stale {
        reason: UiExpressionStaleReason::ProjectionRetained {
            operand: "r".into(),
            kind: worth_ui_query_binding::UiProjectionRetainedActivityKind::Revalidating,
        },
        last_current: Some(UiExpressionCurrentValue::Condition(false)),
    }
}

#[test]
fn a_comment_only_rebind_re_stamps_a_stale_record_without_evaluating_it() {
    let mut session = launch(READY_CONDITION);
    session
        .expressions
        .retain_outcome("ex.ready", retained_stale());
    let retained = session.expression_record("ex.ready").unwrap().clone();
    let reference = session.expression_result("ex.ready").unwrap();
    assert!(session.is_current_expression_result(&reference));
    let completion = evaluate(&mut session, "ex.ready");
    let before = session.expression_work_counters();

    evidence_only_rebind(&mut session, COMMENTED_CONDITION);

    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.generation(), &session.active_generation_identity());
    assert_ne!(record.generation(), retained.generation());
    assert_ne!(
        record.span(),
        retained.span(),
        "the record carries the new span"
    );
    assert_eq!(
        record.outcome(),
        &retained_stale(),
        "the stale posture keeps its last current value"
    );
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
    assert_eq!(record.program_identity(), retained.program_identity());
    let work = since(&session, before);
    assert_eq!(
        (work.evaluations, work.settled_without_evaluation),
        (0, 0),
        "a re-stamped record is not evaluated again"
    );
    assert_eq!(
        work.operand_probes, 1,
        "re-stamping re-proved the one operand the record read"
    );
    assert!(
        !session.is_current_expression_result(&reference),
        "a reference to the retired generation is no longer current"
    );
    assert_eq!(
        complete(&mut session, completion),
        UiExpressionCompletionReceipt::Stale,
        "a ticket of the retired generation is no longer current"
    );
}

#[test]
fn an_expression_meaning_edit_rebuilds_that_expression_with_its_new_outcome() {
    let mut session = launch(READY_CONDITION);
    session
        .expressions
        .retain_outcome("ex.ready", retained_stale());
    let retained = session.expression_record("ex.ready").unwrap().clone();
    let reference = session.expression_result("ex.ready").unwrap();
    let before = session.expression_work_counters();

    evidence_only_rebind(&mut session, NEGATED_CONDITION);

    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.generation(), &session.active_generation_identity());
    assert_ne!(record.program_identity(), retained.program_identity());
    assert_eq!(
        record.outcome(),
        &UiExpressionOutcome::Condition(true),
        "`!r` over a false `app.ready`, with nothing kept from the old program"
    );
    assert_eq!(record.outcome_revision(), 1, "a rebuilt record starts over");
    let work = since(&session, before);
    assert_eq!(work.evaluations, 1);
    assert_eq!(work.published_changes, 1);
    assert!(!session.is_current_expression_result(&reference));
    let reference = session.expression_result("ex.ready").unwrap();
    assert!(session.is_current_expression_result(&reference));
}

#[test]
fn a_re_stamped_reader_re_settles_when_its_rebuilt_upstream_changes() {
    let mut session = launch(&format!("{READY_CONDITION}\n{READER}"));
    assert_eq!(
        session.expression_record("ex.reader").unwrap().outcome(),
        &UiExpressionOutcome::Condition(false)
    );
    let before = session.expression_work_counters();

    evidence_only_rebind(&mut session, &format!("{NEGATED_CONDITION}\n{READER}"));

    assert_eq!(
        session.expression_record("ex.ready").unwrap().outcome(),
        &UiExpressionOutcome::Condition(true)
    );
    let reader = session.expression_record("ex.reader").unwrap();
    assert_eq!(reader.generation(), &session.active_generation_identity());
    assert_eq!(
        reader.outcome(),
        &UiExpressionOutcome::Condition(true),
        "the reader keeps its program, so it is re-stamped, and then re-settles \
         once its rebuilt upstream changes"
    );
    let reference = session.expression_result("ex.reader").unwrap();
    assert!(session.is_current_expression_result(&reference));
    assert_eq!(
        since(&session, before).evaluations,
        2,
        "the rebuilt `ex.ready`, then its reader through the dependency index"
    );
}

#[test]
fn a_cutover_that_resets_a_fact_the_record_read_rebuilds_that_record() {
    let mut session = launch(READY_CONDITION);
    session
        .update_intent_boolean_fact(
            &UiIntentApplicationFact::boolean("app.ready").unwrap(),
            true,
        )
        .unwrap();
    assert_eq!(
        session.expression_record("ex.ready").unwrap().outcome(),
        &UiExpressionOutcome::Condition(true)
    );
    let predecessor = session.active_generation_identity();
    let work = SuccessionWork::read(&session);

    cut_over(&mut session, READY_CONDITION);

    assert_ne!(session.active_generation_identity(), predecessor);
    assert_pointer_is_fresh(&session, false);
    assert_standing_is_fresh(&session, 0);
    assert_owners_follow(&session, false);
    assert_eq!(
        work.since(&session),
        SuccessionWork {
            reobservations: 0,
            operand_probes: 3,
            index_hits: 0,
            evaluations: 1,
            appearance_batches: 0,
        },
        "the replacement pipeline and its unmounted cutover (W3)"
    );
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.generation(), &session.active_generation_identity());
    assert_eq!(
        record.outcome(),
        &UiExpressionOutcome::Condition(false),
        "the successor fact owner starts from the registered `false`, so the \
         record read a fact its owner no longer holds and is rebuilt"
    );
}

#[test]
fn an_owner_that_missed_a_generation_change_holds_nothing_current() {
    let mut session = launch(READY_CONDITION);
    let launched = session.active_generation_identity();
    let catalog = Arc::clone(
        session
            .application
            .prepared_authority()
            .expression_catalog(),
    );
    let mut missed =
        UiExpressionRuntimeState::activate(catalog, &session_inputs(&session, &launched));
    let slot = missed.catalog().slot_of("ex.ready").unwrap();

    cut_over(&mut session, READY_CONDITION);
    let active = session.active_generation_identity();
    assert_ne!(active, launched, "the cutover committed a successor");
    let successor = session.expression_result("ex.ready").unwrap();
    assert_eq!(
        successor.outcome_revision(),
        missed.record(slot).unwrap().outcome_revision(),
        "only the generation tells the two records apart"
    );
    let receipt = session
        .update_intent_boolean_fact(
            &UiIntentApplicationFact::boolean("app.ready").unwrap(),
            true,
        )
        .unwrap();
    let completion = evaluate(&mut session, "ex.ready");
    let inputs = session_inputs(&session, &active);
    let before = missed.counters();

    assert!(
        !missed.is_current_result(&successor, &active),
        "an owner still on the retired generation reports nothing current"
    );
    assert!(missed.begin_evaluation(slot, &inputs).is_none());
    assert!(
        missed.invalidate_application(&receipt, &inputs).is_empty(),
        "a missed owner reports no changed condition"
    );
    assert_eq!(missed.counters(), before, "a missed owner settles nothing");
    assert_eq!(
        missed.complete_evaluation(completion, &inputs),
        UiExpressionCompletionReceipt::Stale,
        "a ticket of the active generation whose every operand is current is \
         still not admitted into a missed owner"
    );
    let record = missed.record(slot).unwrap();
    assert_eq!(record.generation(), &launched);
    assert_eq!(record.outcome(), &UiExpressionOutcome::Condition(false));
}
