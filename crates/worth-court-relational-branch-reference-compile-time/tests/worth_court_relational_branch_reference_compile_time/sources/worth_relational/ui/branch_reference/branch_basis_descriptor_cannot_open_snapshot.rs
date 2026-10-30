use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::runtime::RelationalRuntime;

fn cannot_open(runtime: &RelationalRuntime, descriptor: &RelationalBranchBasisDescriptor) {
    let _ = runtime.snapshots().snapshot_for_observation(descriptor);
}

fn main() {}
