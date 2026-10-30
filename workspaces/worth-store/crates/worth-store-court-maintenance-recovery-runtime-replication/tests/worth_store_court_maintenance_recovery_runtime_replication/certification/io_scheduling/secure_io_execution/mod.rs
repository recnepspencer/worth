#[allow(unused_imports)]
use crate::secure_io_execution::*;
use worth_store_io_scheduler::execute_ready_queue_plan;

#[test]
fn secure_frame_queue_execution_consumes_backend_secure_io_preservation() {
    let plan = secure_frame_read_ahead_plan();
    let scope = speculative_scope(&plan);
    let completion = completion_for(&plan, scope);
    let outcome = execute_ready_queue_plan(plan, completion);

    let worth_store_io_scheduler::QueueExecutionOutcome::Executed(executed) = outcome else {
        panic!("secure-frame completion with matching scope should execute");
    };
    assert_eq!(executed.counters().read_ahead_units(), 1);

    let plan = secure_frame_read_ahead_plan();
    let wrong_scope = worth_store_physical_backend::BackendQueueSpeculativeScope::admitted(
        plan.grouping_basis().security_scope_identity(),
        plan.grouping_basis().tenant_scope(),
        worth_store_security::StoreKeyScope::BackupExportEnvelope,
    );
    let completion = completion_for(&plan, wrong_scope);
    let outcome = execute_ready_queue_plan(plan, completion);

    let worth_store_io_scheduler::QueueExecutionOutcome::Violation(violation) = outcome else {
        panic!("secure-frame completion with cross-key speculation must violate");
    };
    assert_eq!(
        violation.cause(),
        worth_store_io_scheduler::queue_execution::QueueExecutionViolationCause::BackendContradictedWitness
    );
    assert_eq!(violation.counters().read_ahead_units(), 1);
    assert_eq!(violation.counters().violation_events(), 1);
}
