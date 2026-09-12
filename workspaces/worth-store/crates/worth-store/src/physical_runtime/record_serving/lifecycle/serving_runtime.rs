use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::{
    instance::{PhysicalStoreInstanceFoundation, PhysicalStoreInstanceParts},
    media_ownership::PhysicalMediaObserver,
    AbortedRuntime, ClosedRuntime, RuntimeIdentity,
};

use super::super::lifecycle::record_observation::PhysicalRecordObserver;
use super::super::{PhysicalRecordReader, RecordPublicationResidueObservation};

#[cfg(feature = "certification-test-authority")]
#[path = "serving_runtime/certification/mod.rs"]
mod certification;
mod physical_work;
mod scrub;

pub struct ServingPhysicalRuntime {
    scrub: crate::physical_runtime::integrity::PhysicalIntegrityScrubOwner,
    parts: PhysicalStoreInstanceParts,
}

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime::record_serving) fn from_admission(
        foundation: PhysicalStoreInstanceFoundation,
    ) -> Result<Self, super::super::RecordServingAdmissionInspectionRequired> {
        match PhysicalStoreInstanceParts::from_record_admission(foundation) {
            Ok(parts) => Ok(Self {
                scrub: crate::physical_runtime::integrity::PhysicalIntegrityScrubOwner::new(),
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

    /// Atomically captures and protects the current root for this acquisition.
    pub fn records(
        &self,
    ) -> Result<PhysicalRecordReader, crate::physical_runtime::PhysicalReadProtectionDenial> {
        let (current_root, protection) = self.parts.publication.capture_read_root()?;
        let read = super::super::CanonicalRecordReadPort::new(
            &self.parts.work_runtime,
            self.parts.core.lifecycle_generation(),
            self.parts.work_admission,
            self.parts.scheduler_admission.clone(),
            self.parts.record_work.clone(),
        );
        let mutation = super::super::CanonicalRecordMutationPort::new(
            &self.parts.work_runtime,
            self.parts.core.lifecycle_generation(),
            self.parts.work_admission,
            self.parts.scheduler_admission.clone(),
            self.parts.record_work.clone(),
        );
        let frame_ports = self.parts.residency.ports().clone();
        let writeback = mutation.frame_writeback_port(frame_ports.clone());
        Ok(PhysicalRecordReader {
            execution: crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
                &self.parts.work_runtime,
                self.parts.core.lifecycle_generation(),
            ),
            store: self.store_identity(),
            format: self.parts.format,
            access: self.parts.access,
            current_root,
            protection,
            generation: self.parts.core.lifecycle_generation(),
            runtime: std::sync::Arc::downgrade(&self.parts.work_runtime),
            lifecycle: self.parts.record_owner.reader(),
            residency: super::super::residency::PhysicalResidencyWorkPort::new(
                frame_ports,
                super::super::residency::frame_loading::CanonicalFrameReadSource::new(read),
                writeback,
                self.parts.core.lifecycle_state(),
            ),
        })
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
