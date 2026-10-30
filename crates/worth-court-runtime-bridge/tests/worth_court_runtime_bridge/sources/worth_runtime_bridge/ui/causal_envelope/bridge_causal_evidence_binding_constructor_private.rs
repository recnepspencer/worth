use worth_runtime_bridge::facade::{
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceBindingClass, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner,
};

fn main() {
    let _ = BridgeCausalEvidenceBinding {
        owner: BridgeCausalEvidenceOwner::RuntimeBridge,
        family: BridgeCausalEvidenceFamily::BridgeRoute,
        binding_class: BridgeCausalEvidenceBindingClass::RetainedBridgeRecord,
        reference_identity: "route".into(),
        retained_record_digest: sealed_authority_placeholder(),
        binding_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
