pub(crate) mod binding;
mod execution;
mod foundational_receipt;
#[cfg(test)]
mod foundational_receipt_tests;
mod frontier_admission;
pub(crate) mod output_commit;
mod performed_receipt;
mod plan;
pub(crate) mod progression;
pub(crate) mod revalidation;
pub(crate) mod source_seed;

#[cfg(test)]
mod progression_contract_tests;

pub use execution::InvalidationTraceRecord;
pub(crate) use execution::{FrontierDiagnosticsProjection, FrontierDiagnosticsSidecar};
pub use foundational_receipt::{
    attach_foundational_invalidation_performance_receipt,
    FoundationalInvalidationPerformanceReceipt, InvalidationFoundationalReceiptDenial,
};
pub use frontier_admission::{
    FrontierEntryClassification, FrontierInclusionBasis, FrontierSeedCause,
    FrontierValidationDecision, InvalidationSeed, InvalidationSeedBatch,
};
pub use performed_receipt::{
    InvalidationExecutionSummary, SignalInvalidationExecutionObservation,
    SignalInvalidationExecutionReceipt,
};
pub(crate) use plan::FrontierPlan;
pub use plan::InvalidationPlanningEstimate;
