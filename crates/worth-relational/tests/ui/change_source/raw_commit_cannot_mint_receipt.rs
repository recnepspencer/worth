use worth_relational::facade::history::CommitId;
use worth_relational::facade::runtime::RelationalRuntime;

fn mint(runtime: &RelationalRuntime) {
    let _ = runtime.mint_change_receipt(CommitId(1), None);
}

fn main() {
    let _ = mint;
}
