use super::*;

#[test]
fn approved_effect_denies_stale_evidence_before_handler_work() {
    let (application, _, instance, proposal, approval, _) = approval_journey("applied", 930);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            940,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 941)
        .expect("approved operation requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            instance.branch(),
            SEED_DIMENSION + 1,
            942,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION + 1)
    );
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
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
        .idempotency(&943_u64)
        .for_workflow_operation(&application, &required)
        .expect("the old requirement still names this exact binding")
        .execute_in_program(application.program_runtime());
    assert!(matches!(
        effect,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(_))
    ));
    let spent = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&943_u64)
        .for_workflow_operation(&application, &required)
        .expect("the same requirement still names the operation")
        .execute_in_program(application.program_runtime());
    assert!(matches!(
        spent,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)
    ));
    assert_eq!(
        read_dimension(runtime, instance.branch()),
        SEED_DIMENSION + 1
    );
}
