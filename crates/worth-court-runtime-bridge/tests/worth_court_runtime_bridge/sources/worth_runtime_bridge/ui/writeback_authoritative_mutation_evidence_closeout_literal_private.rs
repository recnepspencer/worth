use worth_runtime_bridge::facade::{
    BridgeAuthoritativeMutationEvidenceCloseout, BridgeAuthorityEvidenceDeferredBoundary,
    BridgeAuthorityEvidenceReadyCapability, BridgeAuthorityEvidenceVerificationGate,
};

fn main() {
    let _ = BridgeAuthoritativeMutationEvidenceCloseout {
        support_digest: sealed_authority_placeholder(),
        ready_capabilities: Vec::<BridgeAuthorityEvidenceReadyCapability>::new(),
        deferred_boundaries: Vec::<BridgeAuthorityEvidenceDeferredBoundary>::new(),
        verification_gates: Vec::<BridgeAuthorityEvidenceVerificationGate>::new(),
        closeout_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
