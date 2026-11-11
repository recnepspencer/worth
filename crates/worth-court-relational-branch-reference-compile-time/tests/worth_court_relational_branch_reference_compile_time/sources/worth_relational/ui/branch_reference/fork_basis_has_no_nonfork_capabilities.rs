use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;

fn main() {
    let runtime: &RelationalRuntime = supplied();
    let basis: AdmittedRelationalForkSourceBasis = supplied();
    let _ = runtime.begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary());
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
