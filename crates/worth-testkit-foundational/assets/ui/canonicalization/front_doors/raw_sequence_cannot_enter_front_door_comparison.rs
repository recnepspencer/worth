use worth_foundational::{
    canonicalization, CanonicalBasisSequence, CanonicalEquivalenceBasis,
};

fn impossible<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let left: CanonicalBasisSequence = impossible();
    let right: CanonicalBasisSequence = impossible();

    let _ = canonicalization()
        .compare()
        .left(left)
        .right(right)
        .under(CanonicalEquivalenceBasis::ExactCanonicalBasis);
}
