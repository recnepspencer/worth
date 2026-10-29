use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::mvcc::RelationalTransactionIntent;

fn main() {
    let runtime: &RelationalRuntime = todo!();
    let basis: AdmittedRelationalForkSourceBasis = todo!();
    let _ = runtime.begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary());
}
