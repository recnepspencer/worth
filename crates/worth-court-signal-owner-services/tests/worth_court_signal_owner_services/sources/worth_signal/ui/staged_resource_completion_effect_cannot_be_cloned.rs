use worth_signal::facade::StagedResourceCompletionEffect;

fn consume_staged_effect(effect: StagedResourceCompletionEffect) {
    let _ = effect.clone();
}

fn main() {}
