use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::{
    instance::{PhysicalStoreInstanceFoundation, PhysicalStoreInstanceParts},
    media_ownership::PhysicalMediaObserver,
    AbortedRuntime, ClosedRuntime, RuntimeIdentity,
};

use super::super::lifecycle::record_observation::PhysicalRecordObserver;
use super::super::RecordPublicationResidueObservation;

mod blob_claim;
#[cfg(feature = "certification-test-authority")]
#[path = "serving_runtime/certification/mod.rs"]
mod certification;
mod physical_work;
mod record_reader;
mod retirement;
mod scrub;

pub struct ServingPhysicalRuntime {
    scrub: crate::physical_runtime::integrity::PhysicalIntegrityScrubOwner,
    blob_declarations: std::sync::Mutex<()>,
    blob_index_publications: std::sync::Mutex<()>,
    parts: PhysicalStoreInstanceParts,
}

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime::record_serving) fn from_admission(
        foundation: PhysicalStoreInstanceFoundation,
    ) -> Result<Self, super::super::RecordServingAdmissionInspectionRequired> {
        match PhysicalStoreInstanceParts::from_record_admission(foundation) {
            Ok(parts) => Ok(Self {
                scrub: crate::physical_runtime::integrity::PhysicalIntegrityScrubOwner::new(),
                blob_declarations: std::sync::Mutex::new(()),
                blob_index_publications: std::sync::Mutex::new(()),
                parts,
            }),
            Err(failure) => {
                let (identity, terminal, cause) = failure.abort();
                Err(super::super::RecordServingAdmissionInspectionRequired::new(
                    identity,
                    terminal,
                    super::super::RecordBootstrapFailure::SignalConstruction(cause),
                ))
            }
        }
    }

    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.parts.core.runtime_identity()
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.parts
            .work_runtime
            .executor
            .record_serving_media()
            .store_identity()
    }

    pub(in crate::physical_runtime) fn maximum_inline_record_bytes(&self) -> u32 {
        self.parts.format.declaration().page_size().bytes()
    }

    /// Serializes publication plus derived-index catch-up. A second blob
    /// cannot advance the latest-publication watermark past an unindexed one.
    pub(in crate::physical_runtime) fn lock_blob_index_publication(
        &self,
    ) -> std::sync::MutexGuard<'_, ()> {
        self.blob_index_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::physical_runtime) fn maximum_layout_node_bytes(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
    ) -> usize {
        crate::physical_runtime::record_serving::planning::batch_placement::maximum_inline_payload_bytes(
            self.parts.format,
            placement,
        ) as usize
    }

    pub(in crate::physical_runtime) fn registered_btree_family(
        &self,
        family: worth_store_contracts::DurableArtifactFamilyId,
    ) -> Result<
        crate::physical_runtime::artifact_family::RegisteredDerivedFamily,
        crate::physical_runtime::artifact_family::RegisteredFamilyDenial,
    > {
        self.parts.artifact_families.btree(family)
    }

    /// Reports installed owners only after this serving Store has been built.
    /// A health denial does not erase an installed owner from this inventory.
    pub const fn installed_capabilities(
        &self,
    ) -> crate::physical_runtime::InstalledCapabilityStatus {
        crate::physical_runtime::InstalledCapabilityStatus::record_serving_with_layouts()
    }

    /// Borrows the real Store-owned blob ingest and selected-read paths.
    pub fn blobs(
        &self,
    ) -> Result<
        crate::physical_runtime::PhysicalBlobFacade<'_>,
        crate::physical_runtime::BlobFacadeDenial,
    > {
        if self.parts.work_runtime.health.requires_inspection() {
            return Err(crate::physical_runtime::BlobFacadeDenial::ServingRequiresInspection);
        }
        Ok(crate::physical_runtime::PhysicalBlobFacade::new(self))
    }

    /// Captures a protected selected root and opens Store-owned derived-index access.
    pub fn layouts(
        &self,
    ) -> Result<
        crate::physical_runtime::PhysicalLayoutAccess<'_>,
        crate::physical_runtime::PhysicalLayoutDenial,
    > {
        if self.parts.work_runtime.health.requires_inspection() {
            return Err(crate::physical_runtime::PhysicalLayoutDenial::ServingRequiresInspection);
        }
        crate::physical_runtime::PhysicalLayoutAccess::from_serving(self)
    }

    /// Serializes the selected object check with durable declaration append.
    /// The selected C5 root, not this mutex, remains the durable claim.
    pub(in crate::physical_runtime) fn lock_blob_declaration(
        &self,
    ) -> std::sync::MutexGuard<'_, ()> {
        self.blob_declarations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Releases only this completed append's reloadable data frames. Other
    /// warm records are not collateral victims of streaming blob ingest.
    pub(in crate::physical_runtime) fn release_blob_ingest_clean_frames(
        &self,
        coordinates: &[worth_store_physical_format::RecordFrameCoordinate],
    ) -> u64 {
        self.parts
            .residency
            .ports()
            .invalidate_completed_clean_frames(coordinates)
    }

    pub fn durability_observation(&self) -> crate::physical_runtime::PhysicalDurabilityObservation {
        debug_assert_eq!(
            self.parts.durability.runtime_identity(),
            self.runtime_identity()
        );
        self.parts.durability.observation()
    }

    pub fn observed_staging_residue(&self) -> bool {
        self.parts.publication.residue().staging_catalog_candidate()
    }

    pub fn observed_non_authoritative_residue(&self) -> bool {
        !self.parts.publication.residue().is_empty()
    }

    pub fn publication_residue(&self) -> RecordPublicationResidueObservation {
        self.parts.publication.residue()
    }

    pub fn physical_mutation_observation(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationObservation {
        self.parts.publication.mutation_observation()
    }

    pub fn media_counters(&self) -> worth_store_physical_backend::MediaCounterSnapshot {
        self.parts
            .work_runtime
            .executor
            .record_serving_media()
            .counters()
    }

    pub fn physical_recovery_journal_counters(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryJournalCounters {
        self.parts.work_runtime.executor.recovery_journal_counters()
    }

    pub const fn root_protocol_counters(
        &self,
    ) -> crate::physical_runtime::RootProtocolRouteCounters {
        self.parts.root_protocol_counters
    }

    pub fn resident_admission_counters(
        &self,
    ) -> crate::physical_runtime::ResidentAdmissionCounters {
        self.parts.residency.ports().resident_integrity_counters()
    }

    /// Returns read-only residency evidence for this serving Store generation.
    ///
    /// The observation exposes admitted limits and executed counters, never
    /// pool, allocation, eviction, retry, dirty, or writeback authority.
    pub fn residency_observation(&self) -> super::super::PhysicalResidencyObservation {
        self.parts
            .residency
            .observation(self.parts.core.lifecycle_generation())
    }

    /// Returns the runtime-bound admission surface for successor physical bytes.
    ///
    /// Recovery, scrub, maintenance, verification, and blob adapters use this
    /// surface to charge temporary operation memory to one exact Store scope.
    /// The returned capability exposes no frame, pool, scheduler, or successor
    /// policy authority.
    pub fn physical_allocations(&self) -> super::super::PhysicalScopedAllocationAdmission<'_> {
        super::super::PhysicalScopedAllocationAdmission::new(
            self.parts.residency.ports(),
            self.parts.core.runtime_identity(),
            self.parts.core.lifecycle_generation(),
        )
    }

    pub fn read_protection_observer(
        &self,
    ) -> crate::physical_runtime::PhysicalReadProtectionObserver {
        self.parts.read_protection.observer()
    }

    pub fn record_submission(&self) -> super::super::PhysicalRecordSubmission {
        super::super::RecordPublicationDirector::submission(&self.parts.publication)
    }

    /// Returns the managed checkpoint submission facade for this Store generation.
    pub fn checkpoints(&self) -> crate::physical_runtime::PhysicalCheckpointSubmission {
        crate::physical_runtime::durability::PhysicalCheckpointRuntimeOwner::submission(
            &self.parts.checkpoint,
        )
    }

    pub(in crate::physical_runtime) fn selected_completed_checkpoint(
        &self,
    ) -> Option<crate::physical_runtime::durability::CompletedDurableCheckpointWitness> {
        self.parts.checkpoint.selected_completed_checkpoint()
    }

    /// Installs a bounded production C4 pause at one physical mutation seam.
    ///
    /// The gate controls scheduling of the ordinary mutation worker; it does
    /// not mint an alternate publication or residency authority.
    pub fn pause_physical_mutation_at(
        &self,
        checkpoint: crate::physical_runtime::production::PhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::production::PhysicalMutationPauseGate {
        self.parts.publication.pause_mutation_at(checkpoint)
    }

    /// Installs a bounded production C4 pause at one checkpoint effect seam.
    pub fn pause_physical_checkpoint_at(
        &self,
        step: crate::physical_runtime::production::PhysicalCheckpointStep,
    ) -> crate::physical_runtime::production::PhysicalCheckpointPauseGate {
        self.parts.checkpoint.pause_at(step)
    }

    pub fn observer(&self) -> PhysicalRecordObserver {
        let (lifecycle, lease) = self.parts.core.media_observation_parts();
        let media = PhysicalMediaObserver::for_record_serving(
            self.runtime_identity(),
            self.store_identity(),
            self.parts
                .work_runtime
                .executor
                .record_serving_media()
                .mutation_owner(),
            self.parts
                .work_runtime
                .executor
                .record_serving_media()
                .profile()
                .clone(),
            self.parts
                .work_runtime
                .executor
                .record_serving_media()
                .counter_observer(),
            lifecycle,
            lease,
        );
        PhysicalRecordObserver::new(
            media,
            self.parts.record_owner.observer(),
            self.parts.format,
            self.parts.publication.current_root().generation(),
            self.parts.publication.residue(),
        )
    }

    pub fn close(self) -> super::super::ServingShutdownOutcome<ClosedRuntime> {
        self.close_plan().execute().into_shutdown()
    }

    pub fn close_plan(self) -> crate::physical_runtime::PhysicalStoreClosePlan {
        drop(self.scrub);
        crate::physical_runtime::PhysicalStoreClosePlan::new(self.parts)
    }

    pub fn abort(self) -> super::super::ServingShutdownOutcome<AbortedRuntime> {
        self.abort_with_evidence().into_shutdown()
    }

    pub fn abort_with_evidence(self) -> crate::physical_runtime::PhysicalStoreAbortOutcome {
        drop(self.scrub);
        crate::physical_runtime::PhysicalStoreAbortOutcome::execute(self.parts)
    }
}
