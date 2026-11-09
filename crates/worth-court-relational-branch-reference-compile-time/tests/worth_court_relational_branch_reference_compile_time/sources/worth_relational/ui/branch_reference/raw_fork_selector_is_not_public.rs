use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn main() {
    let mut runtime: RelationalRuntime = supplied();
    let _ = runtime.fork_branch_from(BranchId("target".to_owned()), &BranchId("main".to_owned()));
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
