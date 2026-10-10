use crate::facade::TruthSnapshotIdentity;
use crate::facade::{
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity, BridgeRequestKind,
    BridgeSignalBranchIdentity, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeSubscriptionDeliveryDensityPosture, BridgeTruthViewSelector, RuntimeBridge,
};
use crate::input::envelope::TruthBranchIdentity;
use crate::speculation::BridgePreviewSessionIdentity;

use super::runtime_fixtures::{activation_ready_for, canonical_consumer};

pub(crate) struct SubscriptionPreviewSessionIdentities {
    pub(crate) declaration_identity: BridgePreviewSessionDeclarationIdentity,
    pub(crate) binding_identity: BridgeSpeculativeBranchBindingIdentity,
    pub(crate) truth_branch_identity: TruthBranchIdentity,
    pub(crate) signal_branch_identity: BridgeSignalBranchIdentity,
    pub(crate) snapshot_identity: TruthSnapshotIdentity,
}

pub(crate) fn preview_declaration(
    identities: &SubscriptionPreviewSessionIdentities,
) -> BridgePreviewSessionDeclaration {
    BridgePreviewSessionDeclaration::new(
        identities.declaration_identity.clone(),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            identities.binding_identity.clone(),
            identities.truth_branch_identity.clone(),
            identities.signal_branch_identity.clone(),
        ),
        crate::facade::BridgePreviewSessionBasis::new(
            BridgeTruthViewSelector::branch_snapshot(
                identities.truth_branch_identity.clone(),
                identities.snapshot_identity.clone(),
            ),
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ]),
            crate::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

pub(crate) fn preview_active_subscription_for(
    runtime: &RuntimeBridge,
    preview_session_identity: BridgePreviewSessionIdentity,
    identities: SubscriptionPreviewSessionIdentities,
    declaration: &crate::facade::BridgeSubscriptionDeclaration,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgePreviewActiveSubscription {
    let ready = activation_ready_for(runtime, declaration, resource_request);
    let admitted_preview = runtime
        .admit_preview_session(preview_session_identity, preview_declaration(&identities))
        .expect("preview session should admit");
    let (active_preview_session, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    let preview_basis = runtime
        .admit_subscription_preview_basis(&active_preview_session, &execution_record)
        .expect("preview basis should admit");
    let cost_profile = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            4,
            1,
        )
        .expect("cost profile should admit");
    runtime.activate_preview_subscription_delivery(
        ready,
        preview_basis,
        cost_profile,
        canonical_consumer(runtime),
    )
}
