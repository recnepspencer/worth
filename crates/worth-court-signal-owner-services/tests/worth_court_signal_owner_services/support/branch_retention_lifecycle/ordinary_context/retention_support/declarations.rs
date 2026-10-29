// Shared fixtures for the Signal exact-retention courts.

pub(crate) use {worth_signal::facade::branch::AdmittedSignalBranchBasis};
pub(crate) use {worth_signal::facade::history::RuntimeBranch as SignalBranchHandle};
pub(crate) use {worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(crate) fn runtime() -> Runtime {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

/// Capture twice on a fresh fork so the first snapshot is genuinely historical:
/// it remains stored, but it is no longer the branch's current target.
pub(crate) fn fork_with_historical_target(
    runtime: &mut Runtime,
) -> (
    SignalBranchHandle,
    AdmittedSignalBranchBasis,
    AdmittedSignalBranchBasis,
) {
    let main_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    let (branch, forked) = runtime
        .fork_signal_branch("retained-history", &main_basis)
        .expect("owner fork should succeed")
        .into_parts();
    let (first_snapshot, historical) = runtime
        .capture_signal_branch_snapshot(&forked)
        .expect("first owner capture should succeed")
        .into_parts();
    let (second_snapshot, current) = runtime
        .capture_signal_branch_snapshot(&historical)
        .expect("second owner capture should succeed")
        .into_parts();
    drop(forked);
    drop(first_snapshot);
    drop(second_snapshot);
    assert_ne!(
        historical.observation().target(),
        current.observation().target(),
        "a second capture must move the branch off the first exact target"
    );
    (branch, historical, current)
}