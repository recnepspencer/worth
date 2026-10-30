use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;

fn main() {
    let runtime: &RelationalRuntime = todo!();
    let basis: AdmittedRelationalForkSourceBasis = todo!();
    let _ = runtime.snapshots().snapshot_for_observation(&basis);
}
