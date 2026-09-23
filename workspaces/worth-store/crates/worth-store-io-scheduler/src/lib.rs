#![forbid(unsafe_code)]

mod backend_capability;
pub mod background_pacing;
mod execution;
pub mod foreground_reservation;
mod interference_accounting;
pub mod queue_execution;
mod resource_envelope;
mod resource_units;
mod security_scope_io;

#[cfg(test)]
mod execution_tests;

pub use backend_capability::{
    admit_backend_capability_for_scheduler_claim,
    admit_backend_capability_for_scheduler_qualified_claim,
    admit_secure_frame_backend_capability_for_scheduler_claim,
    IoSchedulerBackendCapabilityAdmission, IoSchedulerBackendCapabilityDenial,
    IoSchedulerBackendCapabilityRequirement,
};
pub use background_pacing::{
    admit_background_capacity, admit_background_pacing, BackgroundCapacityAdmission,
    BackgroundCapacityAdmissionRequest, BackgroundDebtKind, BackgroundIdleCapacityLease,
    BackgroundIdleCapacityLeaseRequest, BackgroundIoDebt, BackgroundIoPressureClass,
    BackgroundIoPressureShape, BackgroundLeaseRevocation, BackgroundPacingAdmissionBasis,
    BackgroundPacingAdmittedWithDebt, BackgroundPacingCounterSnapshot, BackgroundPacingDeferred,
    BackgroundPacingDenial, BackgroundPacingDenied, BackgroundPacingOutcome,
    BackgroundPacingThrottle, BackgroundPacingViolation, BackgroundPacingYield,
    BackgroundResourceBudget, BackgroundResourceShortfall,
};
#[cfg(any(test, feature = "certification-test-authority"))]
pub use background_pacing::{
    blob_ingest_background_capacity_for_certification_test,
    blob_ingest_deferred_background_capacity_for_certification_test,
    blob_ingest_denied_background_capacity_for_certification_test,
    blob_ingest_page_write_background_capacity_for_certification_test,
    blob_ingest_throttled_background_capacity_for_certification_test,
    blob_ingest_wal_write_background_capacity_for_certification_test,
    blob_ingest_zero_admitted_throttle_background_capacity_for_certification_test,
    checkpoint_flush_wal_background_capacity_for_certification_test,
    execute_background_pressure_for_certification_test,
    mismatched_background_pressure_denial_for_certification_test,
    verification_deferred_background_capacity_for_certification_test,
    verification_denied_background_capacity_for_certification_test,
    verification_throttled_background_capacity_for_certification_test,
    verification_zero_admitted_throttle_background_capacity_for_certification_test,
};
pub use execution::{
    IoQueueCounterSnapshot, IoQueueExecutedEvidenceSource, IoQueueExecutionDenial,
    IoQueueExecutionRecorder,
};
pub use interference_accounting::{
    assess_queue_latency_envelope, BackgroundInterferenceEvidence, InterferenceAttribution,
    InterferenceCounterClaim, InterferenceCounterDenial, InterferenceCounterName,
    InterferenceCounterRequirement, InterferenceCounterRow, InterferenceReplayScope,
    LatencyEnvelopeAssessment, LatencyEnvelopeAssessmentStatus, LatencyEnvelopeClaim,
};
pub use queue_execution::{
    admit_queue_execution_plan, admit_queue_policy_receipt, execute_grouped_ready_queue_plans,
    execute_ready_queue_plan, group_ready_queue_pair, lower_background_queue_lease,
    lower_buffer_pool_read_queue_declaration, lower_buffer_pool_writeback_queue_declaration,
    lower_physical_foreground_work, lower_wal_queue_declaration, AdmittedQueueExecutionPlan,
    BackgroundDispatchAttempt, ExecutedQueueEvidence, ForegroundDispatchTurn, OwedBackgroundTurn,
    PhysicalDispatchSelection, PhysicalForegroundWorkDeclaration, QueueBackpressureCause,
    QueueDurabilityClass, QueueExecutedPlan, QueueExecutionAdmissionDenial,
    QueueExecutionAdmissionRequest, QueueExecutionBackpressured, QueueExecutionCounterSnapshot,
    QueueExecutionDenied, QueueExecutionOutcome, QueueExecutionPlanBinding,
    QueueExecutionProgression, QueueExecutionReadyPlan, QueueExecutionReplayIdentity,
    QueueExecutionViolation, QueueGroupedReadyPlans, QueueGroupingBasis, QueueGroupingDenial,
    QueueGroupingOutcome, QueueGroupingRejected, QueueLocalityIdentity, QueueLocalityRange,
    QueueLocalityRelation, QueuePolicyAdmissionReceipt, QueueReadAheadBasis, QueueRecoveryOrdering,
    QueueWorkClass, QueueWorkDeclaration, QueueWriteBackBasis, QueueWritebackPolicy,
};
pub use resource_envelope::{IoQueueResourceEnvelope, IoQueueResourceEnvelopeDenial};
pub use resource_units::{
    BandwidthToken, CacheResidencyHint, DirtyPageBudget, FlushPermit, IoResourceUnitDenial,
    IoResourceUnitKind, QueueSlot, ReadAheadWindow, ReclaimPermit, SyncDebt, WorkerPermit,
    WriteBackWindow,
};
pub use security_scope_io::{
    admit_secure_io_scope_for_scheduler, admit_security_scope_for_scheduler,
    reject_lower_authority_secure_io_scope_source, IoSchedulerSecurityScopeAdmission,
    IoSchedulerSecurityScopeAdmissionDenial, SchedulerSecurityScopeCapability,
    SecureIoCounterStrength, SecureIoOperation, SecureIoPosture, SecureIoPostureRequirement,
    SecureIoPreservationCounterSnapshot, SecureIoPreservationDenial, SecureIoPreservationReceipt,
    SecureIoPreservationRequest, SecureIoScopeBasis,
};
