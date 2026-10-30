use worth_signal::facade::{SignalGraph, SignalRuntime};

fn main() {
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let branch = runtime.current_branch();
    let _ = runtime.merge_branch(&branch, &branch);
}
