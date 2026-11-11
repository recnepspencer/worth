fn wants_current_basis(
    _value: worth_foundational::CurrentBasisCommittedAuthorityArtifact<&'static str>,
) {
}

fn main() {
    let raw: worth_foundational::FoundationalCommittedAuthorityArtifact<&'static str> = supplied();
    wants_current_basis(raw);
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
