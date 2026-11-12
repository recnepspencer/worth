//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RecoveredPhysicalRuntimeHandoffEvidence>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/handoff/recovered_evidence.rs", "RecoveredPhysicalRuntimeHandoffEvidence", 128 * 1024);
        owner.field::<Option<u64>>("store_rejoin_retained_bytes");
        owner.field::<RecoveredRecoverySessionReceipt>("session");
        owner.field::<ResidentSourceSelection>("selection");
        owner.field::<PhysicalRecoveryDiscoveryCounters>("discovery");
        owner.field::<Vec<PhysicalRecoverySourceDenial>>("root_protocol_denials");
        owner
            .field::<crate::entry::PhysicalRecoveryIntegrityObservations>("integrity_observations");
        owner.field::<StoreRecoveryBindingFreshnessSample>("freshness");
        owner.field::<RecoveryOperationFateSet>("fates");
        owner.field::<RecoveryPlanningCounters>("planning");
        owner.field::<crate::entry::PhysicalRecoveryRootProtocolCounters>("root_protocol_counters");
        owner.field::<RecoveryBaseImagePlan>("base");
        owner.field::<RecoveryQuiescencePlan>("quiescence");
        owner.field::<ClosedRecoveryStagingGeneration>("closed");
        owner.field::<PhysicalRecoveryStagingCounters>("staging");
        owner.field::<PhysicalRecoveryStagingSettlementLedger>("staging_settlements");
        owner.field::<RecoveryPublicationExpectation>("publication_expectation");
        owner.field::<PhysicalRecoveryPublicationCounters>("publication");
        owner.field::<PhysicalRecoveryPublicationSettlementLedger>("publication_settlement");
        owner.field::<PhysicalRecoveryReopenCounters>("reopen");
        owner.field::<RecoveryCleanupPosture>("cleanup");
        owner.field::<crate::integrity_ingress::RecoveryIntegrityIngressTrace>("integrity_trace");
    }
}
