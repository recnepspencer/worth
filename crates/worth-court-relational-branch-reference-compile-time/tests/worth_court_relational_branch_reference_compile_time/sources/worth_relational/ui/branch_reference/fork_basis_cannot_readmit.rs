use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;

fn main() {
    let runtime: &RelationalRuntime = todo!();
    let basis: AdmittedRelationalForkSourceBasis = todo!();
    let _ = runtime.branch_reference_state(&basis);
}
