use worth_signal::facade::StagedHostComputedArtifact;

fn main() {
    let _ = StagedHostComputedArtifact {
        prepared: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
