use worth_signal::facade::core::{
    FrozenResourcePolicyDescriptor, ResourcePolicyDescriptor, ResourcePolicyDigest,
    ResourcePolicySelectionBasis,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _descriptor = FrozenResourcePolicyDescriptor {
        descriptor: fake::<ResourcePolicyDescriptor>(),
        selection_basis: ResourcePolicySelectionBasis::DeclaredName,
        parameter_digest: fake::<ResourcePolicyDigest>(),
        frozen_digest: fake::<ResourcePolicyDigest>(),
    };
}
