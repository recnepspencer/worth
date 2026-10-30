pub(crate) use {std::panic::catch_unwind, std::panic::AssertUnwindSafe};

pub(crate) use worth_signal::facade::branch::SignalBranchSnapshotReconstructionDenial;
pub(crate) use {
    worth_signal::facade::SignalError, worth_signal::facade::SignalGraph,
    worth_signal::facade::SignalRuntime,
};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(crate) fn runtime() -> Runtime {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

pub(crate) fn assert_panics(operation: impl FnOnce()) {
    assert!(catch_unwind(AssertUnwindSafe(operation)).is_err());
}
