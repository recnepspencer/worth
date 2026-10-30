use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::mvcc::RelationalTransactionIntent;

fn main() {
    let runtime: &RelationalRuntime = todo!();
    let descriptor: RelationalBranchBasisDescriptor = todo!();
    let _ = runtime.begin_branch_transaction(
        &descriptor,
        RelationalTransactionIntent::ordinary(),
    );
}
