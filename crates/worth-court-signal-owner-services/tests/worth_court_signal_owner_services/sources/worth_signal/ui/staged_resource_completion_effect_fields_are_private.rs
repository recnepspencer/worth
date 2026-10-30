use worth_signal::facade::{AdmittedResourceCompletion, StagedResourceCompletionEffect};

fn forged_admitted_completion() -> AdmittedResourceCompletion {
    loop {}
}

fn main() {
    let _ = StagedResourceCompletionEffect {
        admitted_completion: forged_admitted_completion(),
    };
}
