use crate::runtime::expression::{
    UiExpressionCompletion, UiExpressionCompletionReceipt, UiExpressionInputs,
    UiExpressionRuntimeState,
};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

/// The owners of `session` an expression reads, with `active` as the
/// generation the session runs.
pub(in crate::facade::entry) fn session_inputs<'session>(
    session: &'session super::WorthUiActiveApplicationSession,
    active: &'session WorthUiActiveApplicationGenerationIdentity,
) -> UiExpressionInputs<'session> {
    UiExpressionInputs {
        generation: active,
        mounted: &session.mounted,
        facts: &session.intent_application_facts,
    }
}

/// Runs the production begin and evaluate steps of `state` for `identity`
/// and returns the completion, unadmitted.
pub(in crate::facade::entry) fn evaluate_in(
    state: &mut UiExpressionRuntimeState,
    inputs: &UiExpressionInputs<'_>,
    identity: &str,
) -> UiExpressionCompletion {
    let slot = state.catalog().slot_of(identity).unwrap();
    let ticket = state
        .begin_evaluation(slot, inputs)
        .expect("the expression is installed and its owner follows the active generation");
    state.evaluate_ticket(ticket)
}

/// Runs the production begin and evaluate steps for `identity` and returns
/// the completion, unadmitted.
pub(in crate::facade::entry) fn evaluate(
    session: &mut super::WorthUiActiveApplicationSession,
    identity: &str,
) -> UiExpressionCompletion {
    let active = session.active_generation_identity();
    let inputs = UiExpressionInputs {
        generation: &active,
        mounted: &session.mounted,
        facts: &session.intent_application_facts,
    };
    evaluate_in(&mut session.expressions, &inputs, identity)
}

/// Admits `completion` through the production owner against the session's
/// active generation and current facts.
pub(in crate::facade::entry) fn complete(
    session: &mut super::WorthUiActiveApplicationSession,
    completion: UiExpressionCompletion,
) -> UiExpressionCompletionReceipt {
    let active = session.active_generation_identity();
    let inputs = UiExpressionInputs {
        generation: &active,
        mounted: &session.mounted,
        facts: &session.intent_application_facts,
    };
    session.expressions.complete_evaluation(completion, &inputs)
}

/// Runs a live evidence-only rebind on `session` whose authored source is the
/// launch component plus `declarations`.
pub(in crate::facade::entry) fn evidence_only_rebind(
    session: &mut super::WorthUiActiveApplicationSession,
    declarations: &str,
) {
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
            crate::runtime::WorthUiSourceProvider::in_memory("expression-evidence-successor")
                .with_file(
                    "app/main.wui",
                    crate::runtime::tests::expression::session_fixture::source(declarations),
                ),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "expression-evidence-successor",
            )],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let set = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        session.classify_observations(set).unwrap()
    else {
        panic!("the runtime artifact excludes expressions, so the edit is evidence only")
    };
    let plan = session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let prepared = session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(1),
        )
        .unwrap();
    assert!(matches!(
        prepared.execute(1),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));
}
