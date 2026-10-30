pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_execution::*;

#[test]
fn unsigned_execution_settles_atomically_and_same_intent_replays_without_signing() {
    let (application, instance, required) = journey(8);
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
    let outcome = execute(&application, &instance, &required, 8, 91005).unwrap();
    let receipt = match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result } => {
            assert_eq!(result.retention_days, 8);
            receipt
        }
        other => panic!("unsigned effect did not commit: {other:?}"),
    };
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
    // Lost-response retry uses the original requirement and key, not a new operation.
    match execute(&application, &instance, &required, 8, 91005).unwrap() {
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed) => {
            assert_eq!(replayed.outcome_identity(), receipt.outcome_identity());
        }
        other => panic!("same-intent execution did not replay: {other:?}"),
    }
    match advance_instance(&application, instance, 91006).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("the business commit did not settle its native operation: {other:?}"),
    }
}

#[test]
fn legacy_approval_floor_cannot_be_removed_by_authoring_an_unsigned_definition() {
    for denial in [
        definition::<ReviewedSetRetentionBinding>().err(),
        definition::<ExplicitReviewedSetRetentionBinding>().err(),
    ] {
        assert_eq!(
            denial.unwrap().kind(),
            ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority
        );
    }
    assert!(definition::<OccurrenceSetRetentionBinding>().is_ok());
}

#[test]
fn reconstructed_occurrence_only_contract_issues_fresh_execution_authority() {
    let (application, instance, original) = journey(8);
    drop(original);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    application
        .runtime()
        .release_workflow_compilation_for_test();
    let rebuilt = match advance_instance(&application, instance.clone(), 91020).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("retained unsigned contract did not reconstruct: {other:?}"),
    };
    assert!(rebuilt
        .workflow_execution_requirements()
        .approval()
        .is_none());
    assert!(matches!(
        execute(&application, &instance, &rebuilt, 8, 91021).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
    match advance_instance(&application, instance, 91022).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("rebuilt execution did not settle: {other:?}"),
    }
}

#[test]
fn direct_invocation_and_wrong_input_cannot_bypass_occurrence_binding() {
    let (application, instance, required) = journey(8);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let input = |retention_days| SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days,
    };
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .advanced()
            .mutate(input(8))
            .without_source()
            .idempotency(&91007_u64)
            .execute_in_program(
                application.program_runtime(),
                ExecutionAllocationPolicy::SystemAllocation
            ),
        Err(WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition)
    ));
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(input(9))
            .without_source()
            .idempotency(&91008_u64)
            .for_workflow_operation(&application, &required),
        Err(WorthQueryWorkflowOperationBindingDenial::RequirementMismatch)
    ));
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
}

#[test]
fn independently_issued_competing_requirement_cannot_repeat_a_performed_occurrence() {
    let (application, instance, first) = journey(8);
    let competing = match advance_instance(&application, instance.clone(), 91009).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("competing occurrence did not resolve: {other:?}"),
    };
    assert!(matches!(
        execute(&application, &instance, &competing, 8, 91010).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    // The occurrence owns one guarded idempotency identity. A different logical
    // request cannot claim it after the winner, even with a separately issued slot.
    assert!(matches!(
        execute(&application, &instance, &first, 8, 91011).unwrap(),
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
    assert!(matches!(
        advance_instance(&application, instance, 91012).unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
}

pub(crate) mod review;
