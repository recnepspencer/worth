use worth_signal::facade::{history::RuntimeSnapshot, SignalRuntime};

fn try_legacy_restore(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    snapshot: &RuntimeSnapshot,
) {
    let _ = runtime.restore_snapshot(snapshot);
    let _ = runtime.restore_branch_snapshot(runtime.current_branch(), snapshot);
}

fn main() {}
