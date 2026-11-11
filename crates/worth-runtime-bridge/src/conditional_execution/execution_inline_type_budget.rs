//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeConditionalExecutionCounters>(
            "crates/worth-runtime-bridge/src/conditional_execution/execution.rs",
            "BridgeConditionalExecutionCounters",
            128 * 1024,
        );
        owner.field::<usize>("signal_graph_checks");
        owner.field::<usize>("snapshot_admission_attempts");
        owner.field::<usize>("signal_slot_reuse_hits");
        owner.field::<usize>("compute_provider_checks");
        owner.field::<usize>("signal_execution_contacts");
        owner.field::<usize>("observation_baseline_writes");
        owner.field::<usize>("decisions_retained");
        owner.field::<usize>("unrelated_lowering_scans");
    }
    {
        let mut owner = census.owner::<BridgeConditionalReentryCounters>(
            "crates/worth-runtime-bridge/src/conditional_execution/execution.rs",
            "BridgeConditionalReentryCounters",
            128 * 1024,
        );
        owner.field::<usize>("runtime_key_checks");
        owner.field::<usize>("lowering_identity_checks");
        owner.field::<usize>("installed_lowering_lookups");
        owner.field::<usize>("signal_graph_checks");
        owner.field::<usize>("signal_contract_checks");
        owner.field::<usize>("snapshot_identity_checks");
        owner.field::<usize>("query_continuation_rebindings");
        owner.field::<usize>("unrelated_lowering_scans");
    }
    {
        let mut owner = census.owner::<BridgeConditionalExecutionRequest<'static>>(
            "crates/worth-runtime-bridge/src/conditional_execution/execution.rs",
            "BridgeConditionalExecutionRequest",
            128 * 1024,
        );
        owner.field::<&'static std::sync::Arc<BridgeInstalledConditionalLowering>>("lowering");
        owner.field::<&'static str>("query_binding_identity");
        owner.field::<u64>("query_capability_identity");
        owner.field::<&'static str>("snapshot_identity");
        owner.field::<Option<&'static str>>("truth_branch_identity");
        owner.field::<Option<&'static crate::snapshot::TruthSnapshotIdentity>>(
            "bridge_snapshot_identity",
        );
        owner.field::<&'static str>("execution_identity");
        owner.field::<u64>("attempt");
    }
    {
        let mut owner = census.owner::<BridgeConditionalQueryContinuationAdmission<'static>>(
            "crates/worth-runtime-bridge/src/conditional_execution/execution.rs",
            "BridgeConditionalQueryContinuationAdmission",
            128 * 1024,
        );
        owner.field::<&'static std::sync::Arc<BridgeInstalledConditionalLowering>>("lowering");
        owner.field::<&'static str>("query_binding_identity");
        owner.field::<u64>("query_capability_identity");
        owner.field::<&'static str>("signal_snapshot_projection");
        owner.field::<Option<&'static crate::snapshot::TruthSnapshotIdentity>>(
            "bridge_snapshot_identity",
        );
        owner.field::<&'static str>("signal_execution_projection");
        owner.field::<u64>("attempt");
    }
}
