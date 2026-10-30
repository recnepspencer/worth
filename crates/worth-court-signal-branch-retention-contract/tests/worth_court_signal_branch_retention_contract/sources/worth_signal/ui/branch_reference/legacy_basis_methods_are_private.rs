use worth_signal::facade::SignalRuntime;

fn try_legacy_basis(runtime: &mut SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.current_branch_basis_artifact();
}

fn main() {}
