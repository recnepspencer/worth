use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;

fn main() {
    let runtime: &RelationalRuntime = supplied();
    let basis: AdmittedRelationalForkSourceBasis = supplied();
    let _ = runtime.branch_reference_state(&basis);
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
