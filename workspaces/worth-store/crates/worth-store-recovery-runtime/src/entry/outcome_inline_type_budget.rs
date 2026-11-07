//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<PhysicalRecoveryOutcome>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome.rs",
            "PhysicalRecoveryOutcome",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<PhysicalRecoveryPublicationIndeterminate>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome.rs",
            "PhysicalRecoveryPublicationIndeterminate",
            128 * 1024,
        );
        owner.field::<StableStoreIdentity>("store");
        owner.field::<super::super::PhysicalRecoverySessionIdentity>("session");
        owner.field::<super::super::PhysicalRecoveryPublicationCounters>("counters");
        owner.field::<super::super::PhysicalRecoveryPublicationSettlementLedger>("settlement");
        owner.field::<Vec<super::super::PhysicalRecoverySourceDenial>>("root_protocol_denials");
        owner.field::<super::super::PhysicalRecoveryRootProtocolCounters>("root_protocol_counters");
        owner
            .field::<super::super::PhysicalRecoveryIntegrityObservations>("integrity_observations");
        owner.field::<Option<super::super::PhysicalRecoveryReopenFailure>>("reopen");
        owner.field::<Option<RecoveredPhysicalRuntimeConstructionDenial>>("handoff");
        owner.field::<bool>("checkpoint_residue_indeterminate");
        owner.field::<Option<crate::handoff::RecoveryCleanupPosture>>("cleanup");
        owner.field::<u64>("recovery_effects");
        owner.field::<crate::integrity_ingress::RecoveryIntegrityIngressTrace>("integrity_trace");
    }
    {
        let _owner = census.owner::<PhysicalRecoveryBlockKind>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome.rs",
            "PhysicalRecoveryBlockKind",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<PhysicalRecoveryBlockEvidence>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome.rs",
            "PhysicalRecoveryBlockEvidence",
            128 * 1024,
        );
        owner.field::<PhysicalRecoveryDiscoveryCounters>("counters");
        owner.field::<Option<RecoveryPlanningCounters>>("planning_counters");
        owner.field::<Option<super::super::PhysicalRecoveryRootProtocolCounters>>(
            "root_protocol_counters",
        );
        owner.field::<Option<String>>("artifact");
        owner.field::<Option<u64>>("source_generation");
        owner.field::<Option<u64>>("lsn");
        owner.field::<Vec<super::super::PhysicalRecoverySourceDenial>>("source_denials");
        owner
            .field::<super::super::PhysicalRecoveryIntegrityObservations>("integrity_observations");
        owner.field::<Option<PhysicalRecoveryPlanningDenial>>("planning_denial");
        owner.field::<Option<super::super::PhysicalRecoveryStagingCounters>>("staging_counters");
        owner.field::<Option<super::super::PhysicalRecoveryStagingDenial>>("staging_denial");
        owner.field::<Option<super::super::PhysicalRecoveryStagingSettlementLedger>>(
            "staging_settlements",
        );
        owner.field::<Option<super::super::PhysicalRecoveryPublicationCounters>>(
            "publication_counters",
        );
        owner
            .field::<Option<super::super::PhysicalRecoveryPublicationDenial>>("publication_denial");
        owner.field::<Option<super::super::PhysicalRecoveryPublicationSettlementLedger>>(
            "publication_settlements",
        );
        owner.field::<Option<worth_store::physical_runtime::RecoveryCheckpointResidueDenial>>(
            "checkpoint_residue_denial",
        );
        owner.field::<crate::integrity_ingress::RecoveryIntegrityIngressTrace>("integrity_trace");
    }
    {
        let mut owner = census.owner::<PhysicalRecoveryBlock>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome.rs",
            "PhysicalRecoveryBlock",
            128 * 1024,
        );
        owner.field::<PhysicalRecoveryBlockCause>("cause");
        owner.field::<StableStoreIdentity>("store");
        owner.field::<super::super::PhysicalRecoverySessionIdentity>("session");
        owner.field::<PhysicalRecoveryBlockEvidence>("evidence");
        owner.field::<u64>("recovery_effects");
    }
    super::block_cause::inline_type_budget::collect(census);
    super::planning_denial::inline_type_budget::collect(census);
}
