#![doc = include_str!("compile_fail_proofs.md")]

mod admission;
mod dispatch;
mod execution;
#[cfg(test)]
mod grouping_basis_tests;
mod observation;
mod policy;
#[cfg(test)]
mod test_execution;

pub use admission::{
    admit_queue_execution_plan, admit_queue_policy_receipt, QueueExecutionAdmissionRequest,
    QueuePolicyAdmissionReceipt,
};
pub use admission::{
    group_ready_queue_pair, QueueExecutionAdmissionDenial, QueueGroupedReadyPlans,
    QueueGroupingDenial, QueueGroupingOutcome, QueueGroupingRejected,
};
pub use dispatch::{
    BackgroundDispatchAttempt, ForegroundDispatchTurn, OwedBackgroundTurn,
    PhysicalDispatchSelection,
};
pub use execution::{
    execute_grouped_ready_queue_plans, execute_ready_queue_plan, AdmittedQueueExecutionPlan,
    ExecutedQueueEvidence, QueueExecutedPlan, QueueExecutionBackpressured, QueueExecutionDenied,
    QueueExecutionOutcome, QueueExecutionProgression, QueueExecutionReadyPlan,
    QueueExecutionViolation, QueueExecutionViolationCause,
};
pub(crate) use observation::{
    QueueExecutionCounterBasis, QueueExecutionObservation, QueueExecutionUnitCounts,
};
pub use observation::{
    QueueExecutionCounterSnapshot, QueueExecutionPlanBinding, QueueExecutionReplayIdentity,
};
pub use policy::{
    lower_background_queue_lease, lower_buffer_pool_read_queue_declaration,
    lower_buffer_pool_writeback_queue_declaration, lower_physical_foreground_work,
    lower_wal_queue_declaration, PhysicalForegroundWorkDeclaration, QueueBackpressureCause,
    QueueDurabilityClass, QueueGroupingBasis, QueueLocalityIdentity, QueueLocalityRange,
    QueueLocalityRelation, QueueReadAheadBasis, QueueRecoveryOrdering, QueueWorkClass,
    QueueWorkDeclaration, QueueWriteBackBasis, QueueWritebackPolicy,
};
#[cfg(test)]
pub(crate) use test_execution::execute_admitted_queue_plan;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
