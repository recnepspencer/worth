use worth_ui_dsl::{
    WorthUiExpressionOperand, WorthUiExpressionOperandSource, WorthUiRustAuthoredArtifactInput,
};
use worth_ui_host_contract::*;

use crate::facade::entry::active_application_session::expression_session_fixture::{
    complete, evaluate,
};
use crate::facade::entry::active_application_session::succession_characterization::{
    assert_owners_follow, assert_pointer_is_fresh, assert_standing_is_fresh, SuccessionWork,
};
use crate::facade::expression::UiExpressionOutcome;
use crate::runtime::expression::UiExpressionCompletionReceipt;

const MUTABLE_CONDITION: &str = "test.appearance.mutable_condition";

/// The pointer consumer plus a condition over the operability fact whose
/// change makes an evidence-only successor publish a frame.
fn source() -> WorthUiRustAuthoredArtifactInput {
    let module = super::super::fixture::consumer_module(None, 2)
        .try_with_condition(
            MUTABLE_CONDITION,
            [WorthUiExpressionOperand::new(
                "m",
                WorthUiExpressionOperandSource::ApplicationBoolean {
                    fact: super::super::fixture::MUTABLE.to_owned(),
                },
            )],
            "m",
        )
        .unwrap();
    WorthUiRustAuthoredArtifactInput::from_modules([module])
}

#[test]
fn establishing_the_mounted_allocation_moves_the_expression_state_to_its_generation() {
    let role = super::super::fixture::role();
    let (mut session, _host) = super::super::fixture::session_with_source(&role, source());
    let launched = session.active_generation_identity();
    let retained = session
        .expression_record(MUTABLE_CONDITION)
        .unwrap()
        .clone();
    let reference = session.expression_result(MUTABLE_CONDITION).unwrap();
    assert!(session.is_current_expression_result(&reference));
    let before = session.expression_work_counters();
    let _ = super::super::super::mounting_fixture::mount_unestablished(&mut session, 1_000);
    let work = SuccessionWork::read(&session);

    session.establish_native_viewport_allocation().unwrap();

    let established = session.active_generation_identity();
    assert_ne!(
        established, launched,
        "establishing the allocation commits a graph successor generation"
    );
    let record = session.expression_record(MUTABLE_CONDITION).unwrap();
    assert_eq!(record.generation(), &established);
    assert_eq!(record.outcome(), retained.outcome());
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
    assert!(
        !session.is_current_expression_result(&reference),
        "a reference taken before establishment names the retired generation"
    );
    let reference = session.expression_result(MUTABLE_CONDITION).unwrap();
    assert!(session.is_current_expression_result(&reference));
    let after = session.expression_work_counters();
    assert_eq!(
        (after.evaluations, after.settled_without_evaluation),
        (before.evaluations, before.settled_without_evaluation),
        "the same program over the same facts is re-stamped, not evaluated again"
    );
    assert_pointer_is_fresh(&session, false);
    assert_standing_is_fresh(&session, 0);
    assert_owners_follow(&session, false);
    assert_eq!(
        work.since(&session),
        SuccessionWork {
            reobservations: 99,
            operand_probes: 99,
            index_hits: 99,
            evaluations: 99,
            appearance_batches: 99,
        },
        "native mounted establishment (W4) alone"
    );
    let _ = session.shutdown();
}

#[test]
fn an_evidence_only_successor_published_with_its_frame_moves_the_expression_state_to_it() {
    let (mut session, host, surfaces) = super::mounted_world_with(source());
    super::motion(
        &mut session,
        surfaces[0],
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let initial = super::super::prepare(&mut session);
    super::super::publish(&mut session, &host, initial, 2);
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored("expression-successor")
                .with_rust_authored_input(source()),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "expression-successor",
            )],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let set = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        session.classify_observations(set).unwrap()
    else {
        panic!("same authored meaning preserves identity")
    };
    let plan = session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    session
        .update_intent_boolean_fact(
            &super::super::fixture::fact(super::super::fixture::MUTABLE),
            false,
        )
        .unwrap();
    let predecessor = session.active_generation_identity().clone();
    let before = session
        .expression_record(MUTABLE_CONDITION)
        .unwrap()
        .clone();
    assert_eq!(before.outcome(), &UiExpressionOutcome::Condition(false));
    let reference = session.expression_result(MUTABLE_CONDITION).unwrap();
    let completion = evaluate(&mut session, MUTABLE_CONDITION);

    let prepared = session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(3),
        )
        .unwrap();
    assert!(
        prepared.prepared_frame().is_some(),
        "changed operability publishes the successor with its own frame"
    );
    for _ in &surfaces {
        host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        prepared.execute(3),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));

    let successor = session.active_generation_identity().clone();
    assert_ne!(successor, predecessor, "the rebind published a successor");
    let record = session.expression_record(MUTABLE_CONDITION).unwrap();
    assert_eq!(record.generation(), &successor);
    assert_eq!(record.identity(), before.identity());
    assert_eq!(record.outcome(), before.outcome());
    assert!(
        !session.is_current_expression_result(&reference),
        "a reference to the retired generation is no longer current"
    );
    let reference = session.expression_result(MUTABLE_CONDITION).unwrap();
    assert!(session.is_current_expression_result(&reference));
    assert_eq!(
        complete(&mut session, completion),
        UiExpressionCompletionReceipt::Stale
    );
    let _ = session.shutdown();
}
