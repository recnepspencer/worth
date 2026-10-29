pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::commit_boundary::*;

#[test]
fn one_approval_transition_cannot_commit_twice_under_different_client_keys() {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    use worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowAdvanceAdapter;

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
    let selected = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .on_branch(instance.branch())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let principal_binding = runtime
        .installed_schema()
        .principal_binding(
            super::super::document_retention_model::schema::DocumentPrincipalBinding::reference(),
        )
        .unwrap();
    let principal = runtime
        .with_host_advancement(|host_phase| {
            selected
                .resolve_authenticated_principal(
                    &host_phase,
                    &principal_binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let document = runtime
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let operation = runtime
        .installed_schema()
        .installed_operation(SetRetention::reference())
        .unwrap();
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
    };
    let candidate = |key: u64| {
        let admission = runtime
            .with_host_advancement(|host_phase| {
                selected
                    .authorize_operation(
                        &host_phase,
                        &principal,
                        &document,
                        &operation,
                        Default::default(),
                        &scope,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let HandlerResult::Completed(completed) = runtime
            .with_application_advancement(&scope, |phase| {
                runtime.execute_mutation_handler::<ReviewedSetRetentionBinding>(
                    &phase,
                    &reviewed_identities(&key, &input),
                    principal.principal_identity(),
                    admission,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            })
            .expect("the fixture policy admits its handler advancement")
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
    let extend = |idempotency| {
        WorthQueryWorkflowAdvanceAdapter::bind_operation_idempotency(
            idempotency,
            required.transition_identity_bytes(),
        )
    };
    let sibling_key = 974_u64;
    let sibling_admission = runtime
        .with_host_advancement(|host_phase| {
            selected
                .authorize_operation(
                    &host_phase,
                    &principal,
                    &document,
                    &operation,
                    Default::default(),
                    &scope,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let HandlerResult::Completed(sibling) = runtime
        .with_application_advancement(&scope, |phase| {
            runtime.execute_mutation_handler::<
            super::super::document_retention_model::retention_entry::SetRetentionBinding,
        >(&phase,
&ApplicationMutationIdentities::<DocumentRetentionSchema, SetRetentionBinding>::encode(
                &sibling_key,
                &input,
            )
            .unwrap(),
            principal.principal_identity(),
            sibling_admission,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        })
        .expect("the fixture policy admits its handler advancement")
        .unwrap()
    else {
        panic!("the ordinary sibling handler must produce a candidate");
    };
    let sibling_program = runtime
        .with_host_advancement(|host_phase| {
            sibling
        .into_parts()
        .0
        .bind_workflow_operation_authority(
            &host_phase, &authority,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        })
        .expect("installed host call opens");
    let relabeled = application
        .program_runtime()
        .compare_and_commit_program_action(
            sibling_program,
            &reviewed_identities(&sibling_key, &input),
            extend,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        relabeled,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
    let first_candidate = candidate(972);
    let first_program = runtime
        .with_host_advancement(|host_phase| {
            first_candidate
        .bind_workflow_operation_authority(
            &host_phase, &authority,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        })
        .expect("installed host call opens");
    let second_candidate = candidate(973);
    let second_program = runtime
        .with_host_advancement(|host_phase| {
            second_candidate
        .bind_workflow_operation_authority(
            &host_phase, &authority,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        })
        .expect("installed host call opens");
    let first = application
        .program_runtime()
        .compare_and_commit_program_action(
            first_program,
            &reviewed_identities(&972, &input),
            extend,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        first,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let second = application
        .program_runtime()
        .compare_and_commit_program_action(
            second_program,
            &reviewed_identities(&973, &input),
            extend,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        second,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    ));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
}

pub(crate) mod advancement_identity;

pub(crate) mod authenticated_signer;

pub(crate) mod guarded_program;
