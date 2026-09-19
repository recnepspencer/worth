use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryProducerInvariantRequirement,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;

const REQUIRED: &[WorthQueryProducerInvariantRequirement] =
    &[WorthQueryProducerInvariantRequirement::new(
        "uninstalled-producer-rule",
        1,
        0,
        ApplicationInvariantExecutionPoint::CommitBoundary,
    )];

#[test]
fn producer_requirement_denies_before_first_publication_and_leaves_retry_clean() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let candidate = admitted_program(&world, &principal, &account, &request, "published")
        .with_producer_required_invariants(REQUIRED);
    let outcome = world
        .application
        .compare_and_commit_application(candidate, idempotency(231, 231));
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("missing producer rule published a candidate: {outcome:?}");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _predecessor = resolved_account(&world, "open", &live_scope());

    let retry = admitted_program(&world, &principal, &account, &request, "published");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(retry, idempotency(231, 231)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
