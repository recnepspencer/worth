#[allow(unused_imports)] use crate::latency_interference::*;
use {worth_foundational::performance, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceBudgetKind, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformanceWorkClass, worth_foundational::FoundationalPolicyAdmissionReceipt};
use {worth_store_budgets::CounterEvidenceStrength};
use {worth_store_certification::S6LatencyInterferenceEvidence};
use {worth_store_contracts::QueueProducerResourceShape};
use {worth_store_io_scheduler::foreground_reservation::admitted_point_read_reservation_for_certification_test};
use {worth_store_io_scheduler::admit_backend_capability_for_scheduler_claim, worth_store_io_scheduler::admit_queue_execution_plan, worth_store_io_scheduler::admit_secure_io_scope_for_scheduler, worth_store_io_scheduler::admit_security_scope_for_scheduler, worth_store_io_scheduler::assess_queue_latency_envelope, worth_store_io_scheduler::execute_ready_queue_plan, worth_store_io_scheduler::BackgroundResourceBudget, worth_store_io_scheduler::CacheResidencyHint, worth_store_io_scheduler::InterferenceCounterName, worth_store_io_scheduler::InterferenceCounterRequirement, worth_store_io_scheduler::LatencyEnvelopeClaim, worth_store_io_scheduler::QueueExecutionReadyPlan, worth_store_io_scheduler::QueueSlot, worth_store_io_scheduler::ReadAheadWindow, worth_store_io_scheduler::SecureIoOperation, worth_store_io_scheduler::SecureIoPostureRequirement, worth_store_io_scheduler::SecureIoPreservationRequest, worth_store_io_scheduler::WorkerPermit};
use {worth_store_physical_backend::AdmittedBackendCapabilityWitness, worth_store_physical_backend::BackendCapabilityAdmissionRequest, worth_store_physical_backend::BackendCapabilityEvidenceBasis, worth_store_physical_backend::BackendCapabilitySupportSet, worth_store_physical_backend::BackendMediaAssumptionSet, worth_store_physical_backend::BackendQueueExecutionAdaptation, worth_store_physical_backend::BackendQueueExecutionCompletion, worth_store_physical_backend::BackendQueueExecutionPosture, worth_store_physical_backend::BackendQueueSpeculativeScope, worth_store_physical_backend::BackendRebindTriggers, worth_store_physical_backend::BackendTargetProfile, worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority};

#[test]
fn real_scheduler_assessment_certifies_without_laundering_sampled_rows() {
    let evidence = certified_latency_evidence(4);
    let queue_depth = evidence
        .rows()
        .iter()
        .find(|row| row.name() == worth_store_io_scheduler::InterferenceCounterName::QueuePeakDepth)
        .expect("scheduler assessment should carry queue-depth row");

    assert_eq!(queue_depth.value(), 4);
    assert_eq!(queue_depth.strength(), worth_store_budgets::CounterEvidenceStrength::Sampled);
    assert!(!receipt_rows(&evidence)
        .iter()
        .any(|(name, _)| name == "store.s6.queue.peak-depth"));
    assert!(receipt_rows(&evidence)
        .iter()
        .any(|(name, count)| name == "store.s6.queue.submitted-units"
            && *count == point_read_units(point_read_budget())));
    assert_eq!(
        receipt_rows(&evidence).len(),
        evidence
            .rows()
            .iter()
            .filter(|row| row.strength() == worth_store_budgets::CounterEvidenceStrength::Exact)
            .count()
    );
}
#[test]
fn certification_replay_preserves_policy_counter_and_proof_topology() {
    let first = certified_latency_evidence(3);
    let second = certified_latency_evidence(3);

    assert_eq!(first.status(), second.status());
    assert_eq!(first.rows(), second.rows());
    assert_eq!(receipt_rows(&first), receipt_rows(&second));
}
