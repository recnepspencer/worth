#[allow(unused_imports)]
use crate::secure_io_execution::*;
use worth_store_contracts::QueueProducerResourceShape;
use worth_store_io_scheduler::foreground_reservation::admitted_secure_frame_read_reservation_for_certification_test;
use worth_store_io_scheduler::queue_execution::QueueExecutionViolationCause;
use {
    worth_foundational::performance,
    worth_foundational::FoundationalPerformanceAccessPatternPosture,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceBreadthLocalityPosture,
    worth_foundational::FoundationalPerformanceBudgetKind,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::FoundationalPolicyAdmissionReceipt,
};
use {
    worth_store_io_scheduler::admit_queue_execution_plan,
    worth_store_io_scheduler::admit_secure_frame_backend_capability_for_scheduler_claim,
    worth_store_io_scheduler::admit_secure_io_scope_for_scheduler,
    worth_store_io_scheduler::admit_security_scope_for_scheduler,
    worth_store_io_scheduler::execute_ready_queue_plan,
    worth_store_io_scheduler::BackgroundResourceBudget,
    worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    worth_store_io_scheduler::QueueExecutionAdmissionRequest,
    worth_store_io_scheduler::QueueExecutionOutcome,
    worth_store_io_scheduler::QueueExecutionReadyPlan, worth_store_io_scheduler::SecureIoOperation,
    worth_store_io_scheduler::SecureIoPostureRequirement,
    worth_store_io_scheduler::SecureIoPreservationRequest,
};
use {
    worth_store_physical_backend::AdmittedBackendCapabilityWitness,
    worth_store_physical_backend::BackendCapabilityAdmissionRequest,
    worth_store_physical_backend::BackendCapabilityEvidenceBasis,
    worth_store_physical_backend::BackendCapabilitySupportSet,
    worth_store_physical_backend::BackendMediaAssumptionSet,
    worth_store_physical_backend::BackendQueueExecutionAdaptation,
    worth_store_physical_backend::BackendQueueExecutionCompletion,
    worth_store_physical_backend::BackendQueueExecutionPosture,
    worth_store_physical_backend::BackendQueueSpeculativeScope,
    worth_store_physical_backend::BackendRebindTriggers,
    worth_store_physical_backend::BackendTargetProfile,
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority,
};

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
