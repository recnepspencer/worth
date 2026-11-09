use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;

fn main() {
    let runtime: &RelationalRuntime = supplied();
    let descriptor: RelationalBranchBasisDescriptor = supplied();
    let _ = runtime.begin_branch_transaction(&descriptor, RelationalTransactionIntent::ordinary());
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
