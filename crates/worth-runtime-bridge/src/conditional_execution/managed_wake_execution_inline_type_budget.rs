//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeManagedConditionalExecutionRequest<'static>>(
            "crates/worth-runtime-bridge/src/conditional_execution/managed_wake_execution.rs",
            "BridgeManagedConditionalExecutionRequest",
            128 * 1024,
        );
        owner.field::<&'static BridgeManagedDueWake>("due_wake");
        owner.field::<&'static Arc<BridgeInstalledConditionalLowering>>("lowering");
        owner.field::<&'static super::super::BridgeConditionalSignalBasisBinding>("signal_basis");
        owner.field::<&'static str>("query_binding_identity");
        owner.field::<u64>("query_capability_identity");
        owner.field::<&'static str>("snapshot_identity");
        owner.field::<Option<&'static str>>("truth_branch_identity");
        owner.field::<Option<&'static crate::snapshot::TruthSnapshotIdentity>>(
            "bridge_snapshot_identity",
        );
        owner.field::<Option<&'static crate::correspondence::BridgeCorrespondenceDeliveryReceipt>>(
            "triggering_correspondence",
        );
        owner.field::<u64>("attempt");
    }
}
