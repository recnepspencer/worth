use worth_signal::facade::{RawCompletionEnvelope, SignalRuntime};

fn raw_completion() -> RawCompletionEnvelope {
    loop {}
}

fn stage_raw_completion(mut runtime: SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.stage_admitted_resource_completion(raw_completion());
}

fn main() {}
