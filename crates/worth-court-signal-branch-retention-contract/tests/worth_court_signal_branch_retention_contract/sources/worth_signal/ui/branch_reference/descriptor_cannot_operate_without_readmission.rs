use worth_signal::facade::{branch::SignalBranchBasisDescriptor, SignalRuntime};

fn try_fork(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    descriptor: &SignalBranchBasisDescriptor,
) {
    let _ = runtime.fork_signal_branch("forged", descriptor);
}

fn main() {}
