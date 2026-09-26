//! The public program-owner commit door cannot bypass workflow authority.

use super::*;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    declaration::application_operation::ApplicationMutationBinding,
    primary_graph::{
        HandlerResult, WorthQueryApplicationIdempotencyBinding, WorthQueryPrincipalResolutionMode,
    },
};

use super::super::bounded_dimension_model::schema::{
    PartIdentityField, SetPartDimension, SetPartDimensionInput,
};

#[test]
fn guarded_action_cannot_commit_through_public_program_owner_without_workflow_authority() {
    let application =
        super::super::bounded_dimension_model::host::publish_workflow_on_first_program();
    let runtime = application.runtime();
    let branch = application.program_runtime().current_world();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let selected = runtime
        .on_branch(branch)
        .select()
        .expect("branch must select");
    let principal_binding = runtime
        .installed_schema()
        .principal_binding(
            super::super::bounded_dimension_model::schema::PartPrincipalBinding::reference(),
        )
        .expect("principal binding must install");
    let principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("operator must resolve");
    let part = selected
        .resolve_entity(
            PartIdentityField::reference(),
            PART_IDENTITY.to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("part must resolve");
    let operation = runtime
        .installed_schema()
        .installed_operation(SetPartDimension::reference())
        .expect("operation must install");
    let input = SetPartDimensionInput {
        identity: PART_IDENTITY.to_owned(),
        dimension: SEED_DIMENSION + 1,
    };
    let candidate = |key: u64| {
        let admission = selected
            .authorize_operation(&principal, &part, &operation, Default::default(), &scope)
            .expect("ordinary operation admission must succeed");
        let HandlerResult::Completed(completed) = runtime
            .execute_mutation_handler::<ReviewedSetPartDimensionBinding>(
                &input,
                &key,
                principal.principal_identity(),
                admission,
            )
            .expect("handler may prepare a candidate")
        else {
            panic!("handler must produce a candidate");
        };
        completed.into_parts().0
    };
    let key = 950_u64;
    let outcome = application
        .program_runtime()
        .compare_and_commit_program_action::<ReviewedSetPartDimensionBinding>(
            candidate(key),
            WorthQueryApplicationIdempotencyBinding::new(
                ReviewedSetPartDimensionBinding::idempotency_key_identity(&key),
                ReviewedSetPartDimensionBinding::input_identity(&input),
            ),
        );
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_dimension(runtime, branch), SEED_DIMENSION);
    let admission = application
        .program_runtime()
        .admit_program_operation::<SetPartDimension>();
    assert!(matches!(
        admission,
        Err(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_dimension(runtime, branch), SEED_DIMENSION);
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            branch,
            SEED_DIMENSION + 1,
            key + 2,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION + 1),
        "the ordinary binding remains independently usable"
    );
}

#[test]
fn one_approval_transition_cannot_commit_twice_under_different_client_keys() {
    use worth_query_execution::facade::workflow_advance::WorthQueryWorkflowAdvanceAdapter;

    let (application, _, instance, proposal, approval, _) = approval_journey("applied", 960);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            970,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 971).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an approved operation, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let selected = runtime.on_branch(instance.branch()).select().unwrap();
    let principal_binding = runtime
        .installed_schema()
        .principal_binding(
            super::super::bounded_dimension_model::schema::PartPrincipalBinding::reference(),
        )
        .unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let part = selected
        .resolve_entity(
            PartIdentityField::reference(),
            PART_IDENTITY.to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = runtime
        .installed_schema()
        .installed_operation(SetPartDimension::reference())
        .unwrap();
    let input = SetPartDimensionInput {
        identity: PART_IDENTITY.to_owned(),
        dimension: 8,
    };
    let candidate = |key: u64| {
        let admission = selected
            .authorize_operation(&principal, &part, &operation, Default::default(), &scope)
            .unwrap();
        let HandlerResult::Completed(completed) = runtime
            .execute_mutation_handler::<ReviewedSetPartDimensionBinding>(
                &input,
                &key,
                principal.principal_identity(),
                admission,
            )
            .unwrap()
        else {
            panic!("the reviewed handler must produce a candidate");
        };
        completed.into_parts().0
    };
    let authority = required
        .authority_slot()
        .take()
        .expect("the owner issued one operation authority");
    let idempotency = |key: u64| {
        WorthQueryWorkflowAdvanceAdapter::bind_operation_idempotency_raw(
            WorthQueryApplicationIdempotencyBinding::new(
                ReviewedSetPartDimensionBinding::idempotency_key_identity(&key),
                ReviewedSetPartDimensionBinding::input_identity(&input),
            ),
            required.transition_identity_bytes(),
        )
    };
    let sibling_key = 974_u64;
    let sibling_admission = selected
        .authorize_operation(&principal, &part, &operation, Default::default(), &scope)
        .unwrap();
    let HandlerResult::Completed(sibling) = runtime
        .execute_mutation_handler::<
            super::super::bounded_dimension_model::dimension_entry::SetPartDimensionBinding,
        >(
            &input,
            &sibling_key,
            principal.principal_identity(),
            sibling_admission,
        )
        .unwrap()
    else {
        panic!("the ordinary sibling handler must produce a candidate");
    };
    let sibling_program = sibling
        .into_parts()
        .0
        .bind_workflow_operation_authority(&authority)
        .unwrap();
    let relabeled = application
        .program_runtime()
        .compare_and_commit_program_action::<ReviewedSetPartDimensionBinding>(
            sibling_program,
            idempotency(sibling_key),
        );
    assert!(matches!(
        relabeled,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_dimension(runtime, instance.branch()), SEED_DIMENSION);
    let first_program = candidate(972)
        .bind_workflow_operation_authority(&authority)
        .unwrap();
    let second_program = candidate(973)
        .bind_workflow_operation_authority(&authority)
        .unwrap();
    let first = application
        .program_runtime()
        .compare_and_commit_program_action::<ReviewedSetPartDimensionBinding>(
            first_program,
            idempotency(972),
        );
    assert!(matches!(
        first,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let second = application
        .program_runtime()
        .compare_and_commit_program_action::<ReviewedSetPartDimensionBinding>(
            second_program,
            idempotency(973),
        );
    assert!(matches!(
        second,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    ));
    assert_eq!(read_dimension(runtime, instance.branch()), 8);
}
