use worth_signal::facade::{DeniedResourceCompletion, SignalRuntime};

fn stage_denied_completion(
    mut runtime: SignalRuntime<(), (), (), (), ()>,
    denied: DeniedResourceCompletion,
) {
    let _ = runtime.stage_admitted_resource_completion(denied);
}

fn main() {}
