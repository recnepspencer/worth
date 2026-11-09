use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;

fn main() {
    let basis: AdmittedRelationalForkSourceBasis = supplied();
    let _ = basis.clone();
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
