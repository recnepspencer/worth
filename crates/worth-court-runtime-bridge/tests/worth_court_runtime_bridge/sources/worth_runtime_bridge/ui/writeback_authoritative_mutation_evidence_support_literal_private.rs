use worth_runtime_bridge::facade::{
    BridgeAggregateMutationEvidenceDigest, BridgeAuthoritativeMutationEvidenceSupport,
    BridgeMutationEvidenceCarryForwardSection, BridgeMutationEvidenceContinuityFamily,
    BridgeMutationEvidenceExistingTruthBindingFamily, BridgeMutationEvidenceNamingFamily,
    BridgeMutationEvidenceSymbolicTargetReferenceFamily,
};

fn main() {
    let _ = BridgeAuthoritativeMutationEvidenceSupport {
        carry_forward_sections: Vec::<BridgeMutationEvidenceCarryForwardSection>::new(),
        existing_truth_binding_families: Vec::<BridgeMutationEvidenceExistingTruthBindingFamily>::new(),
        symbolic_target_reference_families: Vec::<BridgeMutationEvidenceSymbolicTargetReferenceFamily>::new(),
        naming_mutation_families: Vec::<BridgeMutationEvidenceNamingFamily>::new(),
        continuity_mutation_families: Vec::<BridgeMutationEvidenceContinuityFamily>::new(),
        aggregate_evidence_digests: Vec::<BridgeAggregateMutationEvidenceDigest>::new(),
        support_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
