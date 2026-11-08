//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PublicationPayload>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/orchestration/publication/state.rs", "PublicationPayload", 128 * 1024);
        owner.field::<crate::progression::ReopenedBacking>("reopened_backing");
        owner.field::<crate::progression::NamespaceDurableBacking>("namespace_backing");
        owner.field::<AdmittedPlatformAuthority>("authority");
        owner.field::<super::super::super::RecoveryCoordination>("coordination");
        owner.field::<crate::orchestration::ResidentSourceSelection>("selection");
        owner.field::<crate::progression::PlanningCustody>("custody");
        owner.field::<Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>>(
            "verified_selected_tier_custody",
        );
        owner.field::<crate::progression::PhysicalRecoveryDiscoveryCounters>("discovery");
        owner.field::<Vec<crate::entry::PhysicalRecoverySourceDenial>>("root_protocol_denials");
        owner.field::<crate::progression::RecoveryIntegrityEvidence>("integrity");
        owner.field::<crate::integrity_ingress::RecoveryIntegrityIngressTrace>("integrity_trace");
        owner.field::<worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample>(
            "freshness",
        );
        owner.field::<crate::handoff::RecoveryOperationFateSet>("fates");
        owner.field::<worth_store_recovery_physics::RecoveryPlanningCounters>("planning");
        owner.field::<crate::entry::PhysicalRecoveryRootProtocolCounters>("root_protocol_counters");
        owner.field::<crate::progression::RecoveryBaseImagePlan>("base");
        owner.field::<crate::progression::RecoveryQuiescencePlan>("quiescence");
        owner.field::<crate::progression::ClosedRecoveryStagingGeneration>("closed");
        owner.field::<crate::entry::PhysicalRecoveryStagingCounters>("staging");
        owner.field::<crate::entry::PhysicalRecoveryStagingSettlementLedger>("staging_settlements");
    }
    {
        let mut owner = census.owner::<PublicationState>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/orchestration/publication/state.rs", "PublicationState", 128 * 1024);
        owner.field::<PublicationStateBacking>("backing");
    }
}
