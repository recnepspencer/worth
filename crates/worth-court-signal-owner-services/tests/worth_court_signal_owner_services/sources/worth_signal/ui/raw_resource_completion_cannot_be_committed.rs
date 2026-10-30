use worth_signal::facade::{RawCompletionEnvelope, SignalRuntime};

fn raw_completion() -> RawCompletionEnvelope {
    loop {}
}

fn commit_raw_completion(mut runtime: SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.commit_staged_resource_completion(raw_completion());
}

fn main() {}
