use std::sync::Arc;

use super::expression_session_fixture::{
    complete, evaluate, evaluate_in, evidence_only_rebind, session_inputs,
};
use crate::facade::expression::UiExpressionOutcome;
use crate::facade::intent::UiIntentApplicationFact;
use crate::runtime::expression::{UiExpressionCompletionReceipt, UiExpressionRuntimeState};
use crate::runtime::tests::expression::cutover_fixture::cut_over;
use crate::runtime::tests::expression::session_fixture::launch;

const READY_CONDITION: &str =
    "condition ex.ready { operand r application-boolean app.ready; when (r) }";

#[test]
fn a_ticket_held_across_a_cutover_completes_stale_and_changes_no_record() {
    let mut session = launch(READY_CONDITION);
    let completion = evaluate(&mut session, "ex.ready");

    cut_over(&mut session, READY_CONDITION);
    let retained = session.expression_record("ex.ready").unwrap().clone();
    let before = session.expression_work_counters();
    let receipt = complete(&mut session, completion);

    assert_eq!(receipt, UiExpressionCompletionReceipt::Stale);
    let after = session.expression_work_counters();
    assert_eq!(after.stale_completions, before.stale_completions + 1);
    assert_eq!(after.published_changes, before.published_changes);
    assert_eq!(after.suppressed_unchanged, before.suppressed_unchanged);
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.outcome(), retained.outcome());
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
    assert_eq!(record.generation(), retained.generation());
}

#[test]
fn a_result_reference_is_current_only_for_its_own_generation() {
    let mut session = launch(READY_CONDITION);
    let reference = session.expression_result("ex.ready").unwrap();
    assert!(session.is_current_expression_result(&reference));

    cut_over(&mut session, READY_CONDITION);

    assert!(
        !session.is_current_expression_result(&reference),
        "the successor generation has its own record and revision"
    );
    let successor = session.expression_result("ex.ready").unwrap();
    assert_eq!(
        successor.generation(),
        &session.active_generation_identity()
    );
    assert!(session.is_current_expression_result(&successor));
}

#[test]
fn a_ticket_from_another_session_completes_as_wrong_world() {
    let mut other = launch(READY_CONDITION);
    let completion = evaluate(&mut other, "ex.ready");
    let mut session = launch(READY_CONDITION);
    let retained = session.expression_record("ex.ready").unwrap().clone();
    let before = session.expression_work_counters();

    let receipt = complete(&mut session, completion);

    assert_eq!(receipt, UiExpressionCompletionReceipt::WrongWorld);
    assert_eq!(
        session.expression_work_counters().stale_completions,
        before.stale_completions + 1
    );
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.outcome(), retained.outcome());
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
}

#[test]
fn a_current_ticket_completes_and_an_unchanged_result_keeps_its_revision() {
    let mut session = launch(READY_CONDITION);
    let revision = session
        .expression_record("ex.ready")
        .unwrap()
        .outcome_revision();
    let completion = evaluate(&mut session, "ex.ready");

    let receipt = complete(&mut session, completion);

    assert_eq!(
        receipt,
        UiExpressionCompletionReceipt::Applied { changed: false }
    );
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.outcome(), &UiExpressionOutcome::Condition(false));
    assert_eq!(record.outcome_revision(), revision);
}

#[test]
fn a_span_only_edit_leaves_identity_and_outcome_equal() {
    let compact = launch(READY_CONDITION);
    let shifted = launch(&format!("\n\n    {READY_CONDITION}\n"));

    let (compact, shifted) = (
        compact.expression_record("ex.ready").unwrap(),
        shifted.expression_record("ex.ready").unwrap(),
    );

    assert_ne!(compact.span(), shifted.span(), "the edit moved the body");
    assert!(compact.span().is_some() && shifted.span().is_some());
    assert_eq!(compact.identity(), shifted.identity());
    assert_eq!(compact.program_identity(), shifted.program_identity());
    assert_eq!(compact.outcome(), shifted.outcome());
    assert_eq!(compact.outcome_revision(), shifted.outcome_revision());
}

#[test]
fn a_ticket_held_across_a_fact_update_completes_stale_and_changes_no_record() {
    let mut session = launch(READY_CONDITION);
    let completion = evaluate(&mut session, "ex.ready");
    session
        .update_intent_boolean_fact(
            &UiIntentApplicationFact::boolean("app.ready").unwrap(),
            true,
        )
        .unwrap();
    let retained = session.expression_record("ex.ready").unwrap().clone();
    assert_eq!(retained.outcome(), &UiExpressionOutcome::Condition(true));
    let before = session.expression_work_counters();

    let receipt = complete(&mut session, completion);

    assert_eq!(
        receipt,
        UiExpressionCompletionReceipt::Stale,
        "the ticket read `app.ready` while it was false"
    );
    let after = session.expression_work_counters();
    assert_eq!(after.stale_completions, before.stale_completions + 1);
    assert_eq!(after.published_changes, before.published_changes);
    assert_eq!(after.suppressed_unchanged, before.suppressed_unchanged);
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.outcome(), &UiExpressionOutcome::Condition(true));
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
    assert_eq!(record.operands(), retained.operands());
}

#[test]
fn a_ticket_held_across_an_upstream_change_completes_stale() {
    let declarations = format!(
        "{READY_CONDITION}\ncondition ex.reader {{ operand up condition ex.ready; when (up) }}"
    );
    let mut session = launch(&declarations);
    let completion = evaluate(&mut session, "ex.reader");
    session
        .update_intent_boolean_fact(
            &UiIntentApplicationFact::boolean("app.ready").unwrap(),
            true,
        )
        .unwrap();
    let retained = session.expression_record("ex.reader").unwrap().clone();
    assert_eq!(retained.outcome(), &UiExpressionOutcome::Condition(true));

    let receipt = complete(&mut session, completion);

    assert_eq!(receipt, UiExpressionCompletionReceipt::Stale);
    let record = session.expression_record("ex.reader").unwrap();
    assert_eq!(record.outcome(), retained.outcome());
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
}

#[test]
fn a_live_evidence_only_rebind_moves_the_expression_state_to_the_successor() {
    let mut session = launch(READY_CONDITION);
    let predecessor = session.active_generation_identity().clone();
    let before = session.expression_record("ex.ready").unwrap().clone();
    let reference = session.expression_result("ex.ready").unwrap();
    let completion = evaluate(&mut session, "ex.ready");

    evidence_only_rebind(&mut session, &format!("\n\n    {READY_CONDITION}\n"));

    let successor = session.active_generation_identity().clone();
    assert_ne!(successor, predecessor, "the rebind published a successor");
    let record = session.expression_record("ex.ready").unwrap();
    assert_eq!(record.generation(), &successor);
    assert_ne!(record.generation(), before.generation());
    assert_ne!(
        record.span(),
        before.span(),
        "the record carries the new span"
    );
    assert_eq!(record.identity(), before.identity());
    assert_eq!(record.outcome(), before.outcome());
    assert!(
        !session.is_current_expression_result(&reference),
        "a reference to the retired generation is no longer current"
    );
    let reference = session.expression_result("ex.ready").unwrap();
    assert!(session.is_current_expression_result(&reference));
    assert_eq!(
        complete(&mut session, completion),
        UiExpressionCompletionReceipt::Stale
    );
}

#[test]
fn a_record_lands_in_its_own_slot_when_an_earlier_completion_was_refused_during_activation() {
    let declarations = format!(
        "{READY_CONDITION}\ncondition ex.reader {{ operand up condition ex.ready; when (up) }}"
    );
    let mut session = launch(&declarations);
    let mut state = UiExpressionRuntimeState::unsettled(
        Arc::clone(
            session
                .application
                .prepared_authority()
                .expression_catalog(),
        ),
        session.active_generation_identity().clone(),
        &session.mounted,
    );
    let (base, reader) = (
        state.catalog().slot_of("ex.ready").unwrap(),
        state.catalog().slot_of("ex.reader").unwrap(),
    );
    assert!(base < reader, "the reader ranks after its upstream");
    let active = session.active_generation_identity();
    let refused = evaluate_in(&mut state, &session_inputs(&session, &active), "ex.ready");
    session
        .update_intent_boolean_fact(
            &UiIntentApplicationFact::boolean("app.ready").unwrap(),
            true,
        )
        .unwrap();
    let admit = |state: &mut UiExpressionRuntimeState,
                 session: &super::WorthUiActiveApplicationSession,
                 completion| {
        state.complete_evaluation(completion, &session_inputs(session, &active))
    };

    assert_eq!(
        admit(&mut state, &session, refused),
        UiExpressionCompletionReceipt::Stale,
        "the fact moved after the upstream read it"
    );
    let held = evaluate_in(&mut state, &session_inputs(&session, &active), "ex.reader");
    let unavailable = evaluate_in(&mut state, &session_inputs(&session, &active), "ex.reader");
    assert_eq!(
        admit(&mut state, &session, unavailable),
        UiExpressionCompletionReceipt::Applied { changed: true }
    );

    assert!(
        state.record(base).is_none(),
        "the refused upstream holds no record, and no other record took its place"
    );
    let record = state.record(reader).expect("the reader is retained");
    assert_eq!(record.identity(), "ex.reader");
    assert_eq!(record.slot(), reader);
    assert!(matches!(
        record.outcome(),
        UiExpressionOutcome::Unavailable(_)
    ));

    let upstream = evaluate_in(&mut state, &session_inputs(&session, &active), "ex.ready");
    assert_eq!(
        admit(&mut state, &session, upstream),
        UiExpressionCompletionReceipt::Applied { changed: true }
    );
    assert_eq!(state.record(base).unwrap().identity(), "ex.ready");
    assert_eq!(
        admit(&mut state, &session, held),
        UiExpressionCompletionReceipt::Stale,
        "the ticket read the upstream as absent; it now holds a record"
    );
}
