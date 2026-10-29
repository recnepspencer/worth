pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial, worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome};
pub(crate) use {worth_signal::facade::runtime::SignalBranchRetirementDenial};
pub(crate) use {worth_signal::facade::SignalBranchRetirementReason, worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime};

pub(crate) fn runtime() -> SignalRuntime<(), (), (), (), ()> {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}