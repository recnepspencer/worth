use worth_signal::facade::{AdmittedResourceCompletion, SignalRuntime};

fn admitted_completion() -> AdmittedResourceCompletion {
    loop {}
}

fn commit_admitted_completion(mut runtime: SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.commit_staged_resource_completion(admitted_completion());
}

fn main() {}
