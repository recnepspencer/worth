use crate::facade::{
    BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis,
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewStructuralBasis, BridgeRequestKind, BridgeSignalBranchIdentity,
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeSpeculativeBranchBinding,
    BridgeSpeculativeBranchBindingIdentity, BridgeTruthViewSelector, SnapshotReadContract,
    TruthBranchIdentity, TruthSnapshotIdentity,
};

mod lifecycle;
mod promotion;
mod replay_and_diagnostics;
mod reuse;

fn preview_declaration() -> BridgePreviewSessionDeclaration {
    BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::admit_bridge_owned("preview:analysis"),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned("binding:analysis"),
            crate::truth_identity_fixtures::truth_branch_fixture("truth:analysis"),
            BridgeSignalBranchIdentity::admit_bridge_owned("signal:analysis"),
        ),
        preview_session_basis(PreviewSessionBasisInput {
            truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture(
                "truth:analysis",
            ),
            snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
                "snapshot:analysis",
            ),
        }),
    )
}

struct PreviewSessionBasisInput {
    truth_branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
}

fn preview_session_basis(input: PreviewSessionBasisInput) -> BridgePreviewSessionBasis {
    BridgePreviewSessionBasis::new(
        BridgeTruthViewSelector::branch_snapshot(
            input.truth_branch_identity,
            input.snapshot_identity,
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ]),
        BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
    )
}

fn structural_basis(revision: u64) -> BridgePreviewStructuralBasis {
    use worth_foundational::facade::{
        AspectContract, AspectContractRevision, AspectIdentity, AspectKey, ScalarAspectType,
    };

    let contract = SnapshotReadContract::new(AspectContract::scalar(
        AspectKey::new("topology").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(revision),
        ScalarAspectType::String,
    ));
    BridgePreviewStructuralBasis::from_snapshot_contract(
        &contract,
        preview_declaration().session_basis().truth_view_selector(),
    )
}
