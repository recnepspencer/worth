use worth_signal::facade::{DeniedResourceCompletion, StagedDeniedResourceCompletionEffect};

fn forged_denied_completion() -> DeniedResourceCompletion {
    loop {}
}

fn main() {
    let _forged = StagedDeniedResourceCompletionEffect {
        denied_completion: forged_denied_completion(),
    };
}
