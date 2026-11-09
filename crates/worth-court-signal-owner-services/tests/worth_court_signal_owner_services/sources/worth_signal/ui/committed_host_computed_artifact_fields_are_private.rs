use worth_signal::facade::CommittedHostComputedArtifact;

fn main() {
    let _ = CommittedHostComputedArtifact { staged: supplied() };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
