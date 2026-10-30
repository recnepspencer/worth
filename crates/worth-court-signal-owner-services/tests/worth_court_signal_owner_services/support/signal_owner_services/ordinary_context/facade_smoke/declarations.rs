pub(crate) use {
    worth_signal::facade::branch::validate_signal_branch_name,
    worth_signal::facade::branch::SignalBranchBasisPort,
    worth_signal::facade::branch::SignalBranchLifecyclePort,
    worth_signal::facade::branch::SignalBranchMutationPort,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerLifecycleObservation,
    worth_signal::facade::branch::SignalOwnerServiceIssuanceDenial,
    worth_signal::facade::branch::SignalOwnerServicePorts,
    worth_signal::facade::branch::SignalOwnerUnavailable,
    worth_signal::facade::branch::ValidatedSignalBranchName,
};
pub(crate) use {worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;
pub(crate) type Services = SignalOwnerServicePorts<(), (), (), (), ()>;

pub(crate) fn runtime() -> Runtime {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

pub(crate) fn assert_send_sync_clone<T: Send + Sync + Clone>(_: &T) {}
