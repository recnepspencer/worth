use worth_foundational::{derive_canonical_digest, FoundationalProfileIdentity};

fn requires_profile_identity(_: FoundationalProfileIdentity) {}

fn main() {
    let digest = derive_canonical_digest(supplied());
    requires_profile_identity(digest);
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
