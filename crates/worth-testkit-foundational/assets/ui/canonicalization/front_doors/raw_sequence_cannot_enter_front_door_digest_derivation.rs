use worth_foundational::{
    canonicalization, CanonicalBasisSequence, CanonicalDigestAlgorithmId,
};

fn impossible<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let sequence: CanonicalBasisSequence = impossible();

    let _ = canonicalization()
        .digest()
        .for_sequence(sequence, CanonicalDigestAlgorithmId::sha256());
}
