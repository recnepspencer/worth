use worth_runtime_bridge::facade::BridgePreviewStructuralBasis;


fn main() {
    let _basis = BridgePreviewStructuralBasis {
        structural_contract_digest: sealed_authority_placeholder(),
        validated_declaration_digest: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
