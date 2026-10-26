mod admission;
mod custody;
mod evaluation;
mod input;
mod meter;
mod native;
mod owner_context;
pub use owner_context::ExecutionOwnerContext;
mod ordered;
mod panic_boundary;
mod payload_storage;
mod perturbation;
mod port;
mod prepared;
mod retained_result;
mod run;
mod scope;
mod settlement;
pub use retained_result::ExecutionRetainedResult;
pub(crate) use retained_result::{
    run_retained_batch, run_retained_inputs_batch, run_retained_native_batch,
    run_retained_native_sparse_batch, run_retained_sparse_batch, ResultStorageDenial,
};
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
pub(crate) use meter::detach_meter_for_test;
pub(crate) use meter::enter_certification_activity;
pub(crate) use meter::enter_retained_memory;
pub(crate) use meter::has_active_kernel;
pub(crate) use meter::run_invoking_scope;
pub(crate) use meter::MemoryActivityGuard;
pub use meter::NestedComputationStop;
pub(crate) use meter::RunLimits;
pub(crate) use meter::StructuralSpanFailure;
pub use meter::{
    ExecutionKernelReceipt, ExecutionKernelReceiptStorage, KernelContext, KernelFailure, KernelStop,
};
pub(crate) use ordered::{
    ordered_memory_requirement, run_ordered, run_ordered_until, OrderedOutcome, OrderedStep,
};
pub use port::BatchStop;
pub(crate) use port::{
    run_checked_batch, run_checked_batch_prepared, run_checked_batch_taking,
    run_checked_batch_with_charge, run_owned_batch_taking, BackendKind, BatchOutcome,
};
pub(crate) use prepared::{prepare_batch_resources, PreparedBatchResources};
pub(crate) use scope::{
    run_scope, run_scope_holding, run_scope_measured, run_scope_native_within, run_scope_within,
    ScopeHold,
};
pub use scope::{ScopeOutcome, ScopeStop};

pub(crate) use meter::active_serial_memory;
pub(crate) use meter::active_stop;
pub(crate) use meter::available_serial_memory_bytes;

mod checkpoint_observation;
pub use checkpoint_observation::ExecutionCheckpointObservationForTest;

pub(crate) fn record_request_consult_for_test() {
    checkpoint_observation::record(1);
}

mod task_result;

pub(crate) use meter::{record_nested, NestedStopOwner};
pub(crate) use meter::{request_refusal, request_stop};

mod scope_refusal;
pub(crate) use scope_refusal::{run_admitting_scope, ScopeKernelFailure};

pub(crate) use port::run_native_batch_with_charge;
pub(crate) use port::run_owned_batch_perturbed_for_test;
pub(crate) use port::{run_owned_native_batch, run_owned_native_batch_perturbed_for_test};
