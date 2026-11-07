//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalCurrentRootOwner>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/current_root_owner.rs", "PhysicalCurrentRootOwner", 128 * 1024);
        owner.field::<crate::physical_runtime::RuntimeIdentity>("runtime_identity");
        owner.field::<crate::physical_runtime::PhysicalRecoveryAllocationAdmission>(
            "recovery_allocation",
        );
        owner.field::<release_capacity::backing::ReleasePublicationAllocationOwner>(
            "release_allocation",
        );
        owner.field::<Mutex<Option<std::sync::Arc<capture_pause::ReadRootCapturePause>>>>(
            "capture_pause",
        );
        super::record_read_protection_inline_size(&mut owner);
        owner.field::<std::sync::Arc<blob_claim::BlobClaimRegistry>>("blob_claims");
        owner.field::<std::sync::Arc<Mutex<Option<reclaim::ReclaimFenceState>>>>("reclaim");
        owner.field::<std::sync::Arc<Mutex<PhysicalCurrentRootState>>>("state");
        owner.field::<PhysicalRootPublicationTransitionOwner>("transition");
        owner.field::<std::sync::Arc<
            crate::physical_runtime::durability::retention::PhysicalPublicationAdmission,
        >>("publication");
        owner.field::<Mutex<
            HashMap<
                PhysicalMutationIdentity,
                Vec<crate::physical_runtime::durability::retention::CandidateGrowthLease>,
            >,
        >>("rewrite_growth");
        owner.field::<Mutex<
            HashMap<
                PhysicalMutationIdentity,
                Vec<crate::physical_runtime::durability::retention::DisplacedArtifact>,
            >,
        >>("displaced");
    }
    {
        let mut owner = census.owner::<PhysicalCurrentRootState>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/current_root_owner.rs", "PhysicalCurrentRootState", 128 * 1024);
        owner.field::<DurablePhysicalRootManifest>("current_root");
        owner.field::<Option<RetainedPhysicalRoot>>("previous_root");
        owner.field::<crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence>(
            "namespace_evidence",
        );
        owner.field::<DurableFreeSpaceManifestHeader>("free_space");
        owner.field::<certificate_capacity::CheckpointCustodyState>("checkpoint_custody");
        owner.field::<release_capacity::ReleaseLedgerState>("release_ledger");
    }
    {
        let mut owner = census.owner::<CompletedPhysicalRootPublication>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/current_root_owner.rs", "CompletedPhysicalRootPublication", 128 * 1024);
        owner.field::<PhysicalDurabilityGroupBasis>("group");
        owner.field::<Box<[PhysicalRootPublicationMemberIdentity]>>("member_identities");
        owner.field::<NonEmpty<RootPublicationPhysicalMutationMember>>("members");
        owner.field::<DurablePhysicalRootManifest>("current_root");
        owner.field::<Box<[RecordArtifactFile]>>("current_artifacts");
        owner.field::<RetainedPhysicalRoot>("retained_root");
        owner.field::<crate::physical_runtime::RecordRootPlanningObservation>(
            "root_planning_observation",
        );
    }
    {
        let _owner = census.owner::<PhysicalCurrentRootAdvanceOutcome>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/current_root_owner.rs", "PhysicalCurrentRootAdvanceOutcome", 128 * 1024);
    }
    {
        let mut owner = census.owner::<IndeterminatePhysicalCurrentRootAdvance>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/current_root_owner.rs", "IndeterminatePhysicalCurrentRootAdvance", 128 * 1024);
        owner.field::<RootNamespaceDurablePhysicalMutationMembers>("durable");
        owner.field::<PhysicalCurrentRootAdvanceFailureCause>("cause");
    }
    {
        let _owner = census.owner::<PhysicalCurrentRootAdvanceFailureCause>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/current_root_owner.rs", "PhysicalCurrentRootAdvanceFailureCause", 128 * 1024);
    }
}
