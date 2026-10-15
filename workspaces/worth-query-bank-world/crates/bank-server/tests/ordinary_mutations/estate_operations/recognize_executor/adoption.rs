//! A non-scope context selector remains usable at the actual adoption successor.
use super::fixture::adoption_recognition_world;
use crate::support::request_scope;
use bank_server::{BankApplicationP1, BankMutationControls};
use worth_query_host::facade::application_entry::{
    WorthQueryBranchAdoptionPublicationOutcome, WorthQuerySourceCompletion,
    WorthQuerySourceOnlyAdvance,
};

#[test]
fn executor_context_selector_survives_unwarmed_program_adoption() {
    let fixture = adoption_recognition_world("recognition-context-adoption");
    let actor = fixture.authenticate_specialist();
    let runtime = &fixture.world.runtime;
    let scope = request_scope();
    let target = runtime
        .supported_program_revision::<BankApplicationP1>()
        .unwrap();
    let request = runtime.request(&actor, &scope);
    let programs = request.on_branch(runtime.current_branch()).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(128).unwrap();
    assert!(matches!(
        runtime
            .request(&actor, &scope)
            .on_branch(runtime.current_branch())
            .programs()
            .publish(prepared),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    // No query or rejected ordinary mutation is used as a diagnostic warmup.
    // Owning advance re-resolves its principal; the separate preauthenticated
    // adoption_context court checks the unwarmed LegalAuthority context directly.
    let input =
        bank_domain::schema::RecognizeEstateExecutor::try_from(fixture.action(fixture.executor))
            .unwrap();
    let operation = runtime
        .mutate(bank_server::mutations::recognize_estate_executor(input))
        .as_principal(&actor)
        .controls(BankMutationControls::new(
            scope.clone(),
            bank_domain::proposals::BankIdempotencyKey::new("recognition-successor").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("construction: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) =
        operation.advance(&runtime.request(&actor, &scope))
    else {
        panic!("actual context selector must authorize and complete")
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    completed
        .close_with_evidence(&runtime.request(&actor, &scope))
        .unwrap_or_else(|(_, cause)| panic!("lawful cleanup: {cause:?}"));
    // Independent authorized product read, after the effect, supplies the oracle.
    assert_eq!(super::estate_executors(&fixture), [fixture.executor]);
    assert!(super::estate_authority_is_still_recognized(&fixture));
}
