fn wants_transition_readiness(
    _value: worth_foundational::FoundationalTransitionProductionTestReadyArtifact,
) {
}

fn main() {
    let raw: worth_foundational::FoundationalTransitionBundle<&'static str> = supplied();
    wants_transition_readiness(raw);
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
