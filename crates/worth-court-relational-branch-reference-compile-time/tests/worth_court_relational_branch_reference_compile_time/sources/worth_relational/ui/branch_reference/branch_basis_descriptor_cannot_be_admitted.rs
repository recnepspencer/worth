use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor,
};

fn cannot_admit(descriptor: RelationalBranchBasisDescriptor) -> AdmittedRelationalBranchBasis {
    descriptor.into()
}

fn main() {}
