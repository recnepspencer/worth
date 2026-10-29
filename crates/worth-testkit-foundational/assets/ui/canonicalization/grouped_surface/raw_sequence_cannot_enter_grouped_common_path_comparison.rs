use worth_foundational::canonicalization_api::{
    common_path,
    lower_lane::{basis, comparison},
};

fn impossible<T>() -> T {
    panic!("compile-fail fixture should not run")
}

fn main() {
    let raw: basis::CanonicalBasisSequence = impossible();
    let _ = common_path::canonicalization()
        .compare()
        .left(raw)
        .right(impossible())
        .under(comparison::CanonicalEquivalenceBasis::ExactCanonicalBasis);
}
