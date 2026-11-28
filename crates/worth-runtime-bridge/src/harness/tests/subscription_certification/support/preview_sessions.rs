use crate::facade::TruthSnapshotIdentity;
use crate::facade::{
    BridgePreviewBranchBinding, BridgeSignalBranchIdentity,
    BridgeSubscriptionDeliveryDensityPosture, RuntimeBridge,
};
use crate::input::envelope::TruthBranchIdentity;
use crate::preview::BridgePreviewSessionIdentity;

use super::runtime_fixtures::{activation_ready_for, canonical_consumer};

pub(crate) struct SubscriptionPreviewSessionIdentities {
    pub(crate) truth_branch_identity: TruthBranchIdentity,
    pub(crate) signal_branch_identity: BridgeSignalBranchIdentity,
    pub(crate) snapshot_identity: TruthSnapshotIdentity,
}

pub(crate) fn preview_active_subscription_for(
    runtime: &RuntimeBridge,
    preview_session_identity: BridgePreviewSessionIdentity,
    identities: SubscriptionPreviewSessionIdentities,
    declaration: &crate::facade::BridgeSubscriptionDeclaration,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgePreviewActiveSubscription {
    let ready = activation_ready_for(runtime, declaration, resource_request);
    let active_preview = runtime.begin_preview(
        preview_session_identity,
        BridgePreviewBranchBinding::new(
            identities.truth_branch_identity,
            identities.signal_branch_identity,
        ),
        identities.snapshot_identity,
    );
    let preview_basis = runtime.admit_subscription_preview_basis(&active_preview);
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
