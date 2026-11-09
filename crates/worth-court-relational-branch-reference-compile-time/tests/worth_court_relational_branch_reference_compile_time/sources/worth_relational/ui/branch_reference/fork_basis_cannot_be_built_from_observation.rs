use worth_foundational::FoundationalBranchReferenceObservation;
use worth_relational::facade::branch::{AdmittedRelationalForkSourceBasis, RelationalBranchTarget};

fn main() {
    let observation: FoundationalBranchReferenceObservation<RelationalBranchTarget> = supplied();
    let _basis: AdmittedRelationalForkSourceBasis = observation.into();
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
