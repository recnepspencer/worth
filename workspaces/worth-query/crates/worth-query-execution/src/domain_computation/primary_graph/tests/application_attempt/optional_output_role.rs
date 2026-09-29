//! An at-most-one output role through the installed mutation handler lane:
//! the handler may leave it unbound or bind it once, both commit and read back
//! as `None` and `Some`, a second binding or a binding through an exactly-one
//! token is refused, and an exactly-one role left unbound is still missing.

use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerResult, WorthQueryCompletedMutationCandidate,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    AuthorizationWorld, IdentityExecutionSchema, OptionalOutputInput,
    OptionalOutputMutationBinding, OptionalOutputOperation, OptionalOutputPlan,
    COMPANION_AS_REQUIRED_OUTPUT, COMPANION_OUTPUT, SUBJECT_OUTPUT,
};
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputProjectionDenial,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

#[test]
fn an_unbound_at_most_one_role_commits_without_that_output() {
    let outputs = committed_outputs(OptionalOutputPlan::RequiredOnly);

    assert!(outputs.entity(SUBJECT_OUTPUT).is_ok());
    assert!(
        outputs
            .entity(COMPANION_OUTPUT)
            .expect("an absent optional output is a value, not a denial")
            .is_none(),
        "the unbound optional role reads as None"
    );
    assert_eq!(
        outputs.workflow_content_identity(),
        committed_outputs(OptionalOutputPlan::RequiredOnly).workflow_content_identity(),
        "an absent optional output has one stable canonical form"
    );
}

#[test]
fn a_bound_at_most_one_role_commits_with_that_output() {
    let outputs = committed_outputs(OptionalOutputPlan::RequiredAndOptional);

    assert_eq!(
        outputs
            .entity(COMPANION_OUTPUT)
            .unwrap()
            .map(|companion| companion.entity_id()),
        Some(outputs.entity(SUBJECT_OUTPUT).unwrap().entity_id())
    );
    assert_ne!(
        outputs.workflow_content_identity(),
        committed_outputs(OptionalOutputPlan::RequiredOnly).workflow_content_identity(),
        "a present optional output is distinct from an absent one"
    );
}

#[test]
fn a_second_binding_of_an_at_most_one_role_is_refused() {
    let Err(MutationHandlerExecutionDenial::Handler(denial)) =
        executed(OptionalOutputPlan::RequiredAndOptionalTwice)
    else {
        panic!("a second binding of an at-most-one role must be refused");
    };
    let denial = denial
        .downcast::<WorthQueryApplicationAttemptDenial>()
        .expect("the refusal is the attempt's typed denial");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole
    );
    assert_eq!(denial.subject(), "companion");
}

#[test]
fn an_optional_role_read_through_an_exactly_one_token_is_refused() {
    for plan in [
        OptionalOutputPlan::RequiredOnly,
        OptionalOutputPlan::RequiredAndOptional,
    ] {
        assert_eq!(
            committed_outputs(plan)
                .entity(COMPANION_AS_REQUIRED_OUTPUT)
                .err(),
            Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch),
            "{plan:?}"
        );
    }
}

#[test]
fn an_optional_role_bound_through_an_exactly_one_token_is_refused() {
    let Err(MutationHandlerExecutionDenial::Handler(denial)) =
        executed(OptionalOutputPlan::RequiredAndOptionalAsRequired)
    else {
        panic!("an at-most-one role bound through an exactly-one token must be refused");
    };
    let denial = denial
        .downcast::<WorthQueryApplicationAttemptDenial>()
        .expect("the refusal is the attempt's typed denial");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleCardinalityMismatch
    );
    assert_eq!(denial.subject(), "companion");
}

#[test]
fn an_unbound_exactly_one_role_is_still_missing() {
    let Err(MutationHandlerExecutionDenial::Attempt(denial)) =
        executed(OptionalOutputPlan::OptionalOnly)
    else {
        panic!("a completed candidate must bind every exactly-one role");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );
    assert_eq!(denial.subject(), "subject");
}

type Execution = Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<
            IdentityExecutionSchema,
            OptionalOutputMutationBinding,
        >,
        OptionalOutputInput,
    >,
    MutationHandlerExecutionDenial,
>;

fn committed_outputs(plan: OptionalOutputPlan) -> WorthQueryApplicationOutputCorrespondence {
    let world = installed_authorization_world(true);
    let (execution, idempotency) = execute(&world, plan);
    let Ok(HandlerResult::Completed(completed)) = execution else {
        panic!("the handler completes its candidate");
    };
    let (program, _) = completed.into_parts();
    let WorthQueryApplicationCommitOutcome::Committed(committed) = world
        .application
        .compare_and_commit_application(program, idempotency)
    else {
        panic!("the completed candidate commits");
    };
    committed.output_correspondence().clone()
}

fn executed(plan: OptionalOutputPlan) -> Execution {
    execute(&installed_authorization_world(true), plan).0
}

fn execute(
    world: &AuthorizationWorld,
    plan: OptionalOutputPlan,
) -> (Execution, WorthQueryApplicationIdempotencyBinding) {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(OptionalOutputOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let key = "optional-output".to_owned();
    let input = OptionalOutputInput {
        status: "open".to_owned(),
        plan,
    };
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        OptionalOutputMutationBinding,
    >::encode(&key, &input)
    .expect("the request encodes");
    let idempotency = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
    let execution = world
        .application
        .execute_mutation_handler::<OptionalOutputMutationBinding>(
            &identities,
            principal.principal_identity(),
            admission,
        );
    (execution, idempotency)
}
