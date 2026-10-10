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

pub(crate) use admission::execution_memory_requirement;
pub(crate) use admission::execution_memory_requirement_for_lease;
pub(crate) use admission::AdmittedBatch;
pub use admission::BatchDenial;
pub(crate) use meter::enter_certification_activity;
pub(crate) use meter::enter_retained_memory;
pub(crate) use meter::has_active_kernel;
pub(crate) use meter::MemoryActivityGuard;
pub use meter::NestedComputationStop;
pub use meter::{KernelContext, KernelFailure, KernelStop};
pub(crate) use ordered::{run_ordered, run_ordered_until, OrderedOutcome, OrderedStep};
pub use port::BatchStop;
pub(crate) use port::{
    run_checked_batch, run_checked_batch_prepared, run_checked_batch_taking,
    run_checked_batch_with_charge, run_owned_batch_taking, BackendKind, BatchOutcome,
};
pub(crate) use prepared::{prepare_batch_resources, PreparedBatchResources};
pub(crate) use scope::{run_scope, run_scope_holding, run_scope_within, ScopeHold, ScopeStop};

pub(crate) use meter::active_serial_memory;

mod task_result;

pub(crate) use meter::{record_nested, NestedStopOwner};
pub(crate) use meter::{request_refusal, request_stop};

mod scope_refusal;
pub(crate) use scope_refusal::{run_admitting_scope, ScopeKernelFailure};
