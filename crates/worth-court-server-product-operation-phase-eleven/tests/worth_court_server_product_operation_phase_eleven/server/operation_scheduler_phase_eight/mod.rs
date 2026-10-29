#[allow(unused_imports)] use crate::operation_scheduler_phase_eight::ordinary_context::*;
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

use {worth_server::WorthServer, worth_server::WorthServerLoweredOperationPlan, worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationPlannerInput, worth_server::WorthServerOperationRequestInput, worth_server::WorthServerQueryOperation, worth_server::WorthServerSchedulerCancellationDirective, worth_server::WorthServerSchedulerCancellationPosture, worth_server::WorthServerSchedulerConflictDenialCode, worth_server::WorthServerSchedulerFailurePosture};


#[allow(unused_imports)]
use {worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture};
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};

use {crate::operation_scheduler_phase_eight::ordinary_context::phase_eight_helpers::insert_task, crate::operation_scheduler_phase_eight::ordinary_context::phase_eight_helpers::worth_native_request_input};
use {worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_real_mutation::RealMutationWorkspaceProvider};
use {worth_proof::TransitionOutcome};

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
        worth_server::WorthServerSchedulerConflictDenialCode::ConflictingMutationPlan
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
            worth_server::WorthServerSchedulerCancellationDirective::BeforeAdmission { slot_ordinal: 0 },
            worth_server::WorthServerSchedulerCancellationDirective::AfterAdmissionBeforeExecution {
                slot_ordinal: 1,
            },
            worth_server::WorthServerSchedulerCancellationDirective::DuringExecution { slot_ordinal: 2 },
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
        Some(worth_server::WorthServerSchedulerCancellationPosture::BeforeAdmission)
    );
    assert_eq!(
        executed.outcomes()[1].cancellation_posture(),
        Some(worth_server::WorthServerSchedulerCancellationPosture::AfterAdmissionBeforeExecution)
    );
    assert_eq!(
        executed.outcomes()[2].cancellation_posture(),
        Some(worth_server::WorthServerSchedulerCancellationPosture::DuringExecution)
    );
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

#[path = "mutation_scheduler_reports_exact_lane_and_conflict_counters/mod.rs"]
pub(crate) mod mutation_scheduler_reports_exact_lane_and_conflict_counters;
#[path = "refused_basis_read/mod.rs"]
pub(crate) mod refused_basis_read;
