//! Retirement admits only the spec's authoring authority, on the definition's
//! own branch, through the workflow runtime that issued the request.

use worth_query_host::facade::{
    application_entry::WorkflowDefinitionPreparationDenial,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

use super::super::bounded_dimension_model::dimension_entry::PART_IDENTITY;
use super::super::bounded_dimension_model::workflow::{
    WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
};
use super::*;

#[test]
fn retirement_denies_a_foreign_runtime_authority_or_branch_and_keeps_the_definition_current() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            531,
        ),
    );
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);

    let other = publish_workflow_on_first_program();
    let foreign_runtime = runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&532_u64)
        .prepare_workflow_definition_retirement(&other, first.clone());
    assert!(matches!(
        foreign_runtime,
        Err(WorthQueryWorkflowDefinitionRetirementPreparationDenial::RuntimeMismatch)
    ));

    let wrong_authority = runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&533_u64)
        .prepare_workflow_definition_retirement(&application, first.clone());
    expect_attempt_denial(
        wrong_authority.map(|_| ()),
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAuthorityMismatch,
    );

    let main = runtime.current_world();
    let sibling = runtime
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling branch publishes");
    let foreign_branch = runtime
        .request(&principal, &scope)
        .on_branch(sibling)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&534_u64)
        .prepare_workflow_definition_retirement(&application, first.clone());
    expect_attempt_denial(
        foreign_branch.map(|_| ()),
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAffinityMismatch,
    );

    expect_started(start_instance(&application, first, 535));
}

fn expect_attempt_denial(
    result: Result<(), WorthQueryWorkflowDefinitionRetirementPreparationDenial>,
    expected: WorthQueryApplicationAttemptDenialKind,
) {
    match result {
        Err(WorthQueryWorkflowDefinitionRetirementPreparationDenial::DefinitionPreparation(
            WorkflowDefinitionPreparationDenial::Attempt(attempt),
        )) => assert_eq!(attempt.kind(), expected),
        Err(other) => panic!("expected {expected:?}, got {other:?}"),
        Ok(()) => panic!("expected {expected:?}, but the retirement prepared"),
    }
}
