use worth_foundational::FoundationalProfileIdentity;

fn main() {
    let _ = FoundationalProfileIdentity {
        basis: supplied(),
        digest: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
