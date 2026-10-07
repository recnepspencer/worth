use worth_server::{
    WorthServer, WorthServerLoweredOperationPlan, WorthServerOperationFamily,
    WorthServerOperationPlannerInput, WorthServerOperationRequestInput, WorthServerQueryOperation,
    WorthServerSchedulerCancellationDirective, WorthServerSchedulerCancellationPosture,
    WorthServerSchedulerConflictDenialCode, WorthServerSchedulerFailurePosture,
};

#[path = "support/operation_scheduler/phase_eight_helpers.rs"]
mod phase_eight_helpers;

#[path = "support/query_handoff/fixture.rs"]
mod query_handoff_fixture;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;

use phase_eight_helpers::{insert_task, worth_native_request_input};
use query_handoff_fixture::resolve_request_context;
use query_handoff_runtime::RealMutationWorkspaceProvider;
use worth_proof::TransitionOutcome;

#[test]
fn submission_and_product_mutation_order_is_deterministic() {
    let first = execute_interleaved_batch();
    let second = execute_interleaved_batch();
    let first_trace = first.execution_trace();
    let second_trace = second.execution_trace();

    assert_eq!(first.outcomes().len(), 2);
    assert_eq!(first.counters().admitted_submission_slot_count(), 1);
    assert_eq!(first.counters().completed_submission_slot_count(), 1);
    assert_eq!(first.counters().admitted_mutation_slot_count(), 1);
    assert_eq!(first.counters().completed_mutation_slot_count(), 1);
    assert_eq!(
        first.outcomes()[0].slot().scheduler_lane(),
        "query-write".to_string()
    );
    assert_eq!(
        first.outcomes()[1].slot().scheduler_lane(),
        "product-draft:session-alpha:product-draft".to_string()
    );
    assert_eq!(
        first_trace[0].execution_digest(),
        second_trace[0].execution_digest()
    );
    assert_eq!(
        first_trace[1].execution_digest(),
        second_trace[1].execution_digest()
    );
    assert_eq!(first_trace[0].slot_ordinal(), 0);
    assert_eq!(first_trace[0].scheduler_lane(), "query-write");
    assert_eq!(first_trace[1].slot_ordinal(), 1);
    assert_eq!(
        first_trace[1].scheduler_lane(),
        "product-draft:session-alpha:product-draft"
    );
}

#[test]
fn conflicting_mutation_plans_localize_scheduler_denial() {
    let server = scheduler_server();
    let left = lower_product_mutation_plan(
        &server,
        "session-alpha",
        "basis:product-draft-1",
        "task-left",
    );
    let right = lower_product_mutation_plan(
        &server,
        "session-alpha",
        "basis:product-draft-1",
        "task-right",
    );

    let denial = server
        .operation_scheduler()
        .schedule_batch([left, right])
        .expect_err("same-lane product mutations with the same caller basis must deny");

    assert_eq!(
        denial.code(),
        WorthServerSchedulerConflictDenialCode::ConflictingMutationPlan
    );
    assert!(denial
        .detail()
        .contains("product-draft:session-alpha:product-draft"));
    assert_eq!(denial.scheduler_counters().planned_batch_width(), 2);
    assert_eq!(
        denial
            .scheduler_counters()
            .conflicting_mutation_plan_denial_count(),
        1
    );
    let facts = denial
        .facts()
        .expect("conflict denial should expose structured facts");
    assert_eq!(
        facts.scheduler_lane(),
        "product-draft:session-alpha:product-draft"
    );
    assert_eq!(
        facts.requested_basis_digest(),
        Some("basis:product-draft-1")
    );
    assert_eq!(facts.left_slot_ordinal(), Some(0));
    assert_eq!(facts.right_slot_ordinal(), Some(1));
}

#[test]
fn scheduler_cancellation_for_mutation_paths_records_after_admission_only() {
    let server = scheduler_server();
    let executed = server
        .operation_scheduler()
        .schedule_batch([
            lower_submission_plan(&server, None, "task-before"),
            lower_submission_plan(&server, None, "task-after-admission"),
            lower_product_mutation_plan(&server, "session-alpha", "", "task-during"),
        ])
        .expect("ordered mutation plans should schedule")
        .execute_with_cancellation([
            WorthServerSchedulerCancellationDirective::BeforeAdmission { slot_ordinal: 0 },
            WorthServerSchedulerCancellationDirective::AfterAdmissionBeforeExecution {
                slot_ordinal: 1,
            },
            WorthServerSchedulerCancellationDirective::DuringExecution { slot_ordinal: 2 },
        ]);

    assert_eq!(executed.counters().cancelled_before_admission_count(), 1);
    assert_eq!(
        executed
            .counters()
            .cancelled_after_admission_before_execution_count(),
        1
    );
    assert_eq!(executed.counters().cancelled_during_execution_count(), 1);
    assert_eq!(executed.counters().admitted_submission_slot_count(), 1);
    assert_eq!(executed.counters().admitted_mutation_slot_count(), 1);
    assert_eq!(
        executed.outcomes()[0].cancellation_posture(),
        Some(WorthServerSchedulerCancellationPosture::BeforeAdmission)
    );
    assert_eq!(
        executed.outcomes()[1].cancellation_posture(),
        Some(WorthServerSchedulerCancellationPosture::AfterAdmissionBeforeExecution)
    );
    assert_eq!(
        executed.outcomes()[2].cancellation_posture(),
        Some(WorthServerSchedulerCancellationPosture::DuringExecution)
    );
}

#[test]
fn mutation_scheduler_reports_exact_lane_and_conflict_counters() {
    let server = scheduler_server();
    let seed_plan = lower_product_mutation_plan(&server, "session-alpha", "", "task-seed");
    let stale_basis = seed_plan
        .query_handoff()
        .workspace()
        .snapshot_identity()
        .expect("the seed workspace is open")
        .terminal_projection_for_reporting()
        .to_string();
    let executed = server
        .operation_scheduler()
        .schedule_batch([
            seed_plan,
            lower_product_mutation_plan(&server, "session-alpha", &stale_basis, "task-stale"),
            lower_product_mutation_plan(&server, "session-alpha", "", "task-closed"),
        ])
        .expect("product mutation plans should schedule")
        .execute();

    assert_eq!(executed.counters().admitted_mutation_slot_count(), 1);
    assert_eq!(executed.counters().completed_mutation_slot_count(), 1);
    assert_eq!(executed.counters().stale_basis_stop_count(), 1);
    assert_eq!(executed.counters().queue_closed_slot_count(), 1);
    match executed.outcomes()[1]
        .failure_posture()
        .expect("second slot should fail stale")
    {
        WorthServerSchedulerFailurePosture::StaleMutationBasis {
            expected_basis_digest,
            observed_basis_digest,
        } => {
            assert_eq!(expected_basis_digest, &stale_basis);
            assert_ne!(observed_basis_digest, &stale_basis);
        }
        other => panic!("expected stale mutation basis failure, got {other:?}"),
    }
    match executed.outcomes()[2]
        .failure_posture()
        .expect("third slot should close behind the stale failure")
    {
        WorthServerSchedulerFailurePosture::OrderedLaneClosed {
            scheduler_lane,
            failed_slot_ordinal,
        } => {
            assert_eq!(scheduler_lane, "product-draft:session-alpha:product-draft");
            assert_eq!(*failed_slot_ordinal, 1);
        }
        other => panic!("expected ordered lane closure, got {other:?}"),
    }
}

#[test]
fn session_coordination_mutation_lane_executes_through_ordered_scheduler() {
    let server = scheduler_server();
    let executed = server
        .operation_scheduler()
        .schedule_batch([lower_session_coordination_plan(
            &server,
            "session-alpha",
            "task-session-coordination",
        )])
        .expect("session coordination mutation plans should schedule")
        .execute();

    assert_eq!(executed.counters().admitted_mutation_slot_count(), 1);
    assert_eq!(executed.counters().completed_mutation_slot_count(), 1);
    assert_eq!(
        executed.outcomes()[0].slot().scheduler_lane(),
        "product-session:session-alpha:product-session".to_string()
    );
}

fn execute_interleaved_batch() -> worth_server::WorthServerExecutedOperationBatch {
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

fn scheduler_server() -> WorthServer {
    query_handoff_fixture::test_server_with_middleware(
        RealMutationWorkspaceProvider,
        false,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should enable query mutation"),
    )
}

fn lower_submission_plan(
    server: &WorthServer,
    basis_digest: Option<&str>,
    task_identity: &str,
) -> WorthServerLoweredOperationPlan {
    lower_mutation_plan(
        server,
        WorthServerOperationFamily::QueryDirectSubmission,
        "tasks.insert",
        None,
        basis_digest,
        WorthServerQueryOperation::single_mutation("tasks.insert", insert_task(task_identity)),
    )
}

fn lower_product_mutation_plan(
    server: &WorthServer,
    product_session_identity: &str,
    basis_digest: &str,
    task_identity: &str,
) -> WorthServerLoweredOperationPlan {
    lower_mutation_plan(
        server,
        WorthServerOperationFamily::ProductApplicationMutation,
        "editor.apply",
        Some(product_session_identity),
        (!basis_digest.is_empty()).then_some(basis_digest),
        WorthServerQueryOperation::single_mutation("editor.apply", insert_task(task_identity)),
    )
}

fn lower_session_coordination_plan(
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
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductSessionCoordination)
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
        .lower(WorthServerOperationPlannerInput::new(
            operation_admission,
            worth_server::WorthServerQueryHandoffOperation::direct_mutation_execution(
                WorthServerQueryOperation::single_mutation(
                    operation_name,
                    insert_task(task_identity),
                ),
            ),
        ))
        .expect("session coordination plan should lower")
}

fn lower_mutation_plan(
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
    let mut builder = WorthServerOperationRequestInput::builder()
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
        .lower(WorthServerOperationPlannerInput::new(
            operation_admission,
            worth_server::WorthServerQueryHandoffOperation::query_mutation_execution(operation),
        ))
        .expect("mutation plan should lower")
}

fn admit_mutation(
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
        TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted mutation pipeline result, got {other:?}"),
    }
}

fn admit_worth_native_session(
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
        TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted Worth-native session pipeline result, got {other:?}"),
    }
}
