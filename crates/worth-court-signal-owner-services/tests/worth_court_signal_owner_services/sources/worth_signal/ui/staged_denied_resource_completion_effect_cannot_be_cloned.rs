use worth_signal::facade::StagedDeniedResourceCompletionEffect;

fn duplicate(effect: StagedDeniedResourceCompletionEffect) {
    let _ = effect.clone();
}

fn main() {}
