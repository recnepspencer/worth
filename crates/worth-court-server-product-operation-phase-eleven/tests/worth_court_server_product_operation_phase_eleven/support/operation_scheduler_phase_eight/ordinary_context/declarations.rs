#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

pub(crate) use {
    worth_server::WorthServer, worth_server::WorthServerLoweredOperationPlan,
    worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationPlannerInput,
    worth_server::WorthServerOperationRequestInput, worth_server::WorthServerQueryOperation,
    worth_server::WorthServerSchedulerCancellationDirective,
    worth_server::WorthServerSchedulerCancellationPosture,
    worth_server::WorthServerSchedulerConflictDenialCode,
    worth_server::WorthServerSchedulerFailurePosture,
};

#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture;
#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context;
pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime::runtime_real_mutation::RealMutationWorkspaceProvider;
pub(crate) use {
    crate::operation_scheduler_phase_eight::ordinary_context::phase_eight_helpers::insert_task,
    crate::operation_scheduler_phase_eight::ordinary_context::phase_eight_helpers::worth_native_request_input,
};

pub(crate) fn execute_interleaved_batch() -> worth_server::WorthServerExecutedOperationBatch {
    let server = scheduler_server();
    server
        .operation_scheduler()
        .schedule_batch([
            lower_submission_plan(&server, None, "task-query-write"),
            lower_product_mutation_plan(&server, "session-alpha", "", "task-product-write"),
        ])
        .expect("interleaved ordered plans should schedule")
        .execute()
}

pub(crate) fn scheduler_server() -> WorthServer {
    worth_testkit_server::shared_support_query_handoff_fixture::test_server_with_middleware(
        RealMutationWorkspaceProvider,
        false,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should enable query mutation"),
    )
}

pub(crate) fn lower_submission_plan(
    server: &WorthServer,
    basis_digest: Option<&str>,
    task_identity: &str,
) -> WorthServerLoweredOperationPlan {
    lower_mutation_plan(
        server,
        worth_server::WorthServerOperationFamily::QueryDirectSubmission,
        "tasks.insert",
        None,
        basis_digest,
        worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task(task_identity),
        ),
    )
}

pub(crate) fn lower_product_mutation_plan(
    server: &WorthServer,
    product_session_identity: &str,
    basis_digest: &str,
    task_identity: &str,
) -> WorthServerLoweredOperationPlan {
    lower_mutation_plan(
        server,
        worth_server::WorthServerOperationFamily::ProductApplicationMutation,
        "editor.apply",
        Some(product_session_identity),
        (!basis_digest.is_empty()).then_some(basis_digest),
        worth_server::WorthServerQueryOperation::single_mutation(
            "editor.apply",
            insert_task(task_identity),
        ),
    )
}

pub(crate) fn lower_session_coordination_plan(
    server: &WorthServer,
    product_session_identity: &str,
    task_identity: &str,
) -> WorthServerLoweredOperationPlan {
    let operation_name = "editor.apply";
    let admission = admit_worth_native_session(
        server,
        resolve_request_context(server, worth_native_request_input()),
        operation_name,
    );
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(
                    worth_server::WorthServerOperationFamily::ProductSessionCoordination,
                )
                .with_operation_name(operation_name)
                .with_product_session_identity(product_session_identity)
                .with_idempotency_key(format!(
                    "idem-{operation_name}-session-coordination-{product_session_identity}"
                ))
                .build(),
        )
        .expect("session coordination request should admit");
    let operation_admission = server
        .operation_admissions()
        .admit_declared(&admission, &operation_request)
        .expect("session coordination operation admission should admit");
    server
        .operation_planner()
        .lower(worth_server::WorthServerOperationPlannerInput::new(
            operation_admission,
            worth_server::WorthServerQueryHandoffOperation::direct_mutation_execution(
                worth_server::WorthServerQueryOperation::single_mutation(
                    operation_name,
                    insert_task(task_identity),
                ),
            ),
        ))
        .expect("session coordination plan should lower")
}

pub(crate) fn lower_mutation_plan(
    server: &WorthServer,
    family: WorthServerOperationFamily,
    operation_name: &str,
    product_session_identity: Option<&str>,
    basis_digest: Option<&str>,
    operation: WorthServerQueryOperation,
) -> WorthServerLoweredOperationPlan {
    let admission = admit_mutation(
        server,
        resolve_request_context(server, worth_native_request_input()),
        operation_name,
    );
    let mut builder = worth_server::WorthServerOperationRequestInput::builder()
        .with_operation_family(family)
        .with_operation_name(operation_name)
        .with_idempotency_key(format!(
            "idem-{operation_name}-{product_session_identity:?}"
        ));
    if let Some(product_session_identity) = product_session_identity {
        builder = builder.with_product_session_identity(product_session_identity);
    }
    if let Some(basis_digest) = basis_digest {
        builder = builder.with_basis_digest(basis_digest);
    }
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(&admission, builder.build())
        .expect("mutation operation request should admit");
    let operation_admission = server
        .operation_admissions()
        .admit_declared(&admission, &operation_request)
        .expect("mutation operation admission should admit");
    server
        .operation_planner()
        .lower(worth_server::WorthServerOperationPlannerInput::new(
            operation_admission,
            worth_server::WorthServerQueryHandoffOperation::query_mutation_execution(operation),
        ))
        .expect("mutation plan should lower")
}

pub(crate) fn admit_mutation(
    server: &WorthServer,
    resolved: worth_server::WorthServerResolvedRequestContext,
    operation_name: &str,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_mutation(operation_name),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted mutation pipeline result, got {other:?}"),
    }
}

pub(crate) fn admit_worth_native_session(
    server: &WorthServer,
    resolved: worth_server::WorthServerResolvedRequestContext,
    operation_name: &str,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::worth_native_session(operation_name),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted Worth-native session pipeline result, got {other:?}"),
    }
}
