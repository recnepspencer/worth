//! Back never abandons an approved operation before a receipted settlement
//! consumes the approval, matching the custody program adoption reads.

use super::*;

#[test]
fn back_is_refused_while_an_approval_awaits_its_receipted_operation() {
    let (application, _, instance, proposal, required, _) = approval_journey("applied", 1_960);
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        1_970,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected approval to complete, got {other:?}"),
    }
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let navigate = |key: u64| {
        runtime
            .request(&principal, &scope)
            .mutate(
                super::super::bounded_dimension_model::workflow::WorkflowAdvanceIntent {
                    input: super::super::bounded_dimension_model::workflow::WorkflowAdvanceInput {
                        part_identity: PART_IDENTITY.to_owned(),
                    },
                },
            )
            .without_source()
            .idempotency(&key)
            .prepare_workflow_navigate_back(&application, instance.clone())
            .expect("Back request must receive fresh admission")
            .execute()
    };
    let unsettled = |key: u64| {
        matches!(
            navigate(key),
            Err(WorkflowProgressOutcome::PreparationDenied(denial))
                if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled
        )
    };
    assert!(
        unsettled(1_971),
        "Back cannot abandon the approved operation"
    );

    let required = match advance_instance(&application, instance.clone(), 1_972)
        .expect("the approved effect requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    assert!(
        unsettled(1_973),
        "a prepared requirement is not a receipted settlement"
    );
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&1_974_u64)
        .for_workflow_operation(&application, &required)
        .expect("the effect request matches the durable operation requirement")
        .execute_in_program(application.program_runtime())
        .expect("the approved effect request must execute");
    assert!(matches!(
        effect,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    // The receipt consumed the approval, so the guard steps aside and Back
    // meets the ordinary law: it never crosses a performed operation.
    assert!(matches!(
        navigate(1_975),
        Err(WorkflowProgressOutcome::PreparationDenied(denial))
            if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported
                && denial.subject().contains("performed operation")
    ));
}
