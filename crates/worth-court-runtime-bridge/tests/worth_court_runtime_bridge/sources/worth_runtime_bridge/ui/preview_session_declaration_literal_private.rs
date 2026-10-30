use worth_runtime_bridge::facade::{
    BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis,
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity, BridgeRequestKind,
    BridgeSignalBranchIdentity, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeTruthViewSelector, TruthBranchIdentity, TruthSnapshotIdentity,
};


fn main() {
    let session_basis = BridgePreviewSessionBasis::new(
        BridgeTruthViewSelector::branch_snapshot(
            TruthBranchIdentity::new("truth"),
            TruthSnapshotIdentity::new("snapshot"),
        ),
        BridgeSourceCapabilitySet::new(vec![BridgeSourceCapability::SnapshotRead]),
        BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
    );

    let _declaration = BridgePreviewSessionDeclaration {
        declaration_identity: BridgePreviewSessionDeclarationIdentity::new("preview"),
        request_kind: BridgeRequestKind::Preview,
        branch_binding: BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::new("binding"),
            TruthBranchIdentity::new("truth"),
            BridgeSignalBranchIdentity::new("signal"),
        ),
        session_basis,
        request_shape_basis: native_request_shape_basis(),
        structural_basis: None,
        truth_view_basis_digest: sealed_authority_placeholder(),
        structural_basis_digest: None,
        source_capability_digest: sealed_authority_placeholder(),
        request_shape_digest: sealed_authority_placeholder(),
        retained_artifact_schema_digest: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn native_request_shape_basis<T>() -> T {
    panic!("compile-fail fixture never executes")
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
