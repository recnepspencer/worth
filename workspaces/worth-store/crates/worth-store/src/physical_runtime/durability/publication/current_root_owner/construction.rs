//! Install root publication state and its exact bound live-allocation port.

use super::{
    blob_claim, certificate_capacity, release_capacity, CheckpointCustodyOrigin,
    PhysicalCurrentRootOwner, PhysicalCurrentRootState, PhysicalRootPublicationTransitionOwner,
    PreparedRecoveredCheckpointCustody, RetainedPhysicalRoot,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
        checkpoint_custody_origin: CheckpointCustodyOrigin,
        recovered_checkpoint_custody: Option<PreparedRecoveredCheckpointCustody>,
        current_root: DurablePhysicalRootManifest,
        previous_root: Option<DurablePhysicalRootManifest>,
        free_space: DurableFreeSpaceManifestHeader,
        read_protection: Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
        publication: Arc<crate::physical_runtime::durability::PhysicalPublicationAdmission>,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        frame_ports: crate::physical_runtime::record_serving::RecordFramePorts,
        generation: crate::physical_runtime::LifecycleGeneration,
        lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    ) -> Self {
        let blob_claim_capacity = read_protection.acquisition_capacity();
        let checkpoint_custody = certificate_capacity::CheckpointCustodyState::from_origin(
            checkpoint_custody_origin,
            &current_root,
        );
        let owner = Self {
            runtime_identity: runtime.submission.runtime_identity(),
            recovery_allocation,
            release_allocation: release_capacity::backing::ReleasePublicationAllocationOwner::new(
                frame_ports, recovery_allocation, runtime.submission.runtime_identity(),
                generation, lifecycle,
            ),
            #[cfg(feature = "certification-test-authority")]
            capture_pause: Mutex::new(None),
            blob_claims: Arc::new(blob_claim::BlobClaimRegistry::new(blob_claim_capacity)),
            reclaim: Arc::new(Mutex::new(None)),
            read_protection,
            state: Arc::new(Mutex::new(PhysicalCurrentRootState {
                namespace_evidence:
                    crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence::ReopenedCurrentRoot {
                        root: current_root.root_cell(),
                    },
                current_root,
                previous_root: previous_root.map(RetainedPhysicalRoot::from_manifest),
                free_space,
                checkpoint_custody,
                release_ledger: release_capacity::ReleaseLedgerState::from_origin(checkpoint_custody_origin),
            })),
            transition: PhysicalRootPublicationTransitionOwner::new(runtime),
            publication,
            rewrite_growth: Mutex::new(HashMap::new()),
            displaced: Mutex::new(HashMap::new()),
        };
        if let Some(recovered) = recovered_checkpoint_custody {
            owner.install_recovered_checkpoint_custody(recovered);
        }
        owner
    }
}
