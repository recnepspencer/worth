use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;

fn mint(runtime: &RelationalRuntime, basis: AdmittedRelationalForkSourceBasis) {
    let _ = runtime.mint_change_receipt(basis, None);
}

fn main() {
    let _ = mint;
}
