mod admission;
mod custody;
mod evaluation;
mod input;
mod meter;
mod native;
mod ordered;
mod panic_boundary;
mod perturbation;
mod port;
mod prepared;
mod run;
mod scope;
mod settlement;
mod thread_capability;
pub(crate) use thread_capability::HAS_OS_THREADS;

#[cfg(test)]
mod settlement_order_tests;
#[cfg(test)]
mod settlement_serial_order_tests;

pub(crate) use admission::execution_memory_requirement;
pub(crate) use admission::execution_memory_requirement_for_lease;
pub(crate) use admission::AdmittedBatch;
pub use admission::BatchDenial;
pub(crate) use meter::enter_certification_activity;
pub(crate) use meter::enter_retained_memory;
pub(crate) use meter::has_active_kernel;
pub(crate) use meter::MemoryActivityGuard;
pub(crate) use meter::RunLimits;
pub use meter::{KernelContext, KernelFailure, KernelStop};
pub(crate) use ordered::{run_ordered, run_ordered_until, OrderedOutcome, OrderedStep};
pub use port::BatchStop;
pub(crate) use port::{
    run_checked_batch, run_checked_batch_prepared, run_checked_batch_taking,
    run_checked_batch_with_charge, run_owned_batch_taking, BackendKind, BatchOutcome,
};
pub(crate) use prepared::{prepare_batch_resources, PreparedBatchResources};
pub(crate) use scope::{
    run_scope, run_scope_holding, run_scope_with_charge, run_scope_within, ScopeHold, ScopeStop,
};

pub(crate) use meter::active_serial_memory;
pub(crate) use meter::active_stop;

mod checkpoint_observation;
pub use checkpoint_observation::ExecutionCheckpointObservationForTest;

pub(crate) fn record_request_consult_for_test() {
    checkpoint_observation::record(1);
}
