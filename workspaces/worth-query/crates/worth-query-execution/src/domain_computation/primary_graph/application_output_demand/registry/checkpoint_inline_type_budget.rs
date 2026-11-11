//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOutputCheckpoint>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry/checkpoint.rs", "WorthQueryOutputCheckpoint", 128 * 1024);
        owner.field::<crate::domain_computation::primary_graph::application_contribution::PendingProducerPublication>("PublicationPending.owner");
        owner.field::<super::super::PreparedReadyBacking>("PublicationPending.ready_backing");
        owner.field::<WorthQueryApplicationCommitReceipt>("Published.receipt");
        owner.field::<WorthQueryPendingOutputDelivery>("Published.delivery");
        owner.field::<super::super::PreparedReadyBacking>("Published.ready_backing");
        owner.field::<WorthQueryApplicationCommitReceipt>("Delivered.receipt");
        owner.field::<Option<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>>(
            "Delivered.delivery",
        );
        owner.field::<super::super::PreparedReadyBacking>("Delivered.ready_backing");
        owner.field::<super::super::super::WorthQueryRecoveredProducerPublication>(
            "RecoveredPublished.publication",
        );
        owner.field::<WorthQueryPendingOutputDelivery>("RecoveredPublished.delivery");
        owner.field::<super::super::PreparedReadyBacking>("RecoveredPublished.ready_backing");
        owner.field::<super::super::super::WorthQueryRecoveredProducerPublication>(
            "RecoveredDelivered.publication",
        );
        owner.field::<Option<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>>(
            "RecoveredDelivered.delivery",
        );
        owner.field::<super::super::PreparedReadyBacking>("RecoveredDelivered.ready_backing");
    }
    {
        let mut owner = census.owner::<WorthQueryOutputAdvancement>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry/checkpoint.rs", "WorthQueryOutputAdvancement", 128 * 1024);
        owner.field::<WorthQueryOutputDemandDenial>("Stopped.denial");
        owner.field::<Option<WorthQueryOutputClaimIdentity>>("Stopped.interrupted_claim");
    }
}
