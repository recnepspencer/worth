pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome, worth_signal::facade::branch::SignalBranchRetirementBatchDenial};
pub(crate) use {worth_signal::facade::runtime::SignalBranchRetirementDenial};
pub(crate) use {worth_signal::facade::SignalBranchRetirementReason, worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(crate) fn runtime() -> Runtime {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}