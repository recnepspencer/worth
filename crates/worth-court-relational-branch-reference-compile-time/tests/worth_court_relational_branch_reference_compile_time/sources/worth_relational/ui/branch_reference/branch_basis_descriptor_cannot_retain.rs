use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::runtime::RelationalRuntime;

fn cannot_retain(runtime: &RelationalRuntime, descriptor: &RelationalBranchBasisDescriptor) {
    let _ = runtime.retain_component_basis(descriptor);
}

fn main() {}
