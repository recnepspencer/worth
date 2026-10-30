use worth_signal::facade::{SignalRuntime, StagedDeniedResourceCompletionEffect};

fn commit_denied(
    mut runtime: SignalRuntime<(), (), (), (), ()>,
    staged: StagedDeniedResourceCompletionEffect,
) {
    let _ = runtime.commit_staged_resource_completion(staged);
}

fn main() {}
