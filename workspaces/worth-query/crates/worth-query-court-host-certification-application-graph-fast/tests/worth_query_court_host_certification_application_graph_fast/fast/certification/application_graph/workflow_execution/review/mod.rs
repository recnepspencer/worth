pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_execution::review::*;

#[test]
fn explicitly_required_approval_executes_with_its_exact_native_grant() {
    let (application, instance, required) = approved::<ExplicitReviewedSetRetentionBinding>();
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_some());
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ExplicitReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&92012_u64)
        .for_workflow_operation(&application, &required)
        .unwrap()
        .advanced()
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
    match advance_instance(&application, instance, 92013).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("required approval effect did not settle: {other:?}"),
    }
}

#[test]
fn empty_installed_floor_does_not_ignore_an_authored_approval_or_its_revocation() {
    let (application, instance, required) = approved::<OccurrenceSetRetentionBinding>();
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_none());
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let changed = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .advanced()
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: "workflow-approval-grant".to_owned(),
                status: "revoked".to_owned(),
            },
        })
        .without_source()
        .idempotency(&92014_u64)
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(
        matches!(execute(&application, &instance, &required, 8, 92015),
        Err(WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(denial))
            if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable)
    );
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
}

#[test]
fn occurrence_only_floor_still_consumes_the_authored_approval() {
    // approved() first requires the real AwaitingApproval posture, signs it,
    // and obtains the resulting native operation requirement.
    let (application, instance, required) = approved::<OccurrenceSetRetentionBinding>();
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_none());
    let outcome = execute(&application, &instance, &required, 8, 92016).unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
}
