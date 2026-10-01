#[cfg(feature = "certification-test-authority")]
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::{Arc, Mutex, Weak};

use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

use crate::physical_runtime::instance::PhysicalStoreWorkRuntime;

use super::super::{
    residency::{frame_loading::CanonicalFrameReadSource, PhysicalResidencyWorkPort},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, CanonicalRecordMutationPort,
    CanonicalRecordReadPort, RecordAllocationFrontier, RecordFramePorts,
    RecordPublicationResidueObservation,
};

mod arena_allocation;
mod arena_evacuation;
mod arena_evacuation_producer;
mod arena_retirement;
mod artifact_scope;
mod blob_reclaim;
mod blob_reclaim_released;
#[cfg(feature = "certification-test-authority")]
mod certification_hooks;
#[cfg(feature = "certification-test-authority")]
mod certification_submission;
mod checkpoint_custody;
mod durable_data;
mod durable_preparation;
pub(in crate::physical_runtime) mod extent_copy;
mod extent_record_rewrite;
mod group_wal_planning;
mod lifecycle;
mod managed_mutation;
mod manifest_residue;
mod pre_seal_cancellation;
mod publication_retention;
pub(in crate::physical_runtime) use publication_retention::AdmittedPublicationRetention;
mod record_append_fingerprint;
mod retirement;
mod retirement_progression;
mod retirement_release;
pub(in crate::physical_runtime::record_serving) use arena_evacuation::ArenaEvacuationSelection;
pub use arena_evacuation_producer::PhysicalArenaEvacuationPreparationOutcome;
mod release_head_preparation;
mod released_control_placement;
mod reuse_claim;
mod rewrite_anchor;
mod rewrite_pages;
mod rewrite_source_liveness;
mod rewrite_span_selection;
mod root_candidate_execution;
mod root_capture;
mod root_preparation;
mod root_progression;
mod selected_manifest_pins;
mod selected_segment_rewrite;
pub(in crate::physical_runtime) use selected_manifest_pins::{
    SelectedBlobManifestPin, SelectedBlobManifestPinDenial, SelectedBlobManifestPins,
};
mod submission;
mod tier_epoch;
pub(in crate::physical_runtime) use tier_epoch::TierEpochActivationFailure;
mod wal_data_planning;

pub use artifact_scope::{InlineArtifactRewritePlanDenial, PlannedInlineRewriteArtifact};
#[cfg(feature = "certification-test-authority")]
pub use certification_submission::CertificationPhysicalRecordSubmission;
pub use submission::PhysicalRecordSubmission;

pub(in crate::physical_runtime) struct RecordPublicationDirector {
    reader_factory: crate::physical_runtime::record_serving::lifecycle::record_lifecycle::RecordReaderLeaseFactory,
    runtime: Weak<PhysicalStoreWorkRuntime>,
    mutation_identity: crate::physical_runtime::PhysicalMutationSubmission,
    idempotency: crate::physical_runtime::durability::PhysicalMutationIdempotencyRuntimeAuthority,
    durability: crate::physical_runtime::PhysicalDurabilityObservation,
    signal_profile: crate::physical_runtime::PhysicalSignalProfileIdentity,
    security_basis: [u8; 32],
    durability_policy_basis: crate::physical_runtime::PhysicalWorkSemanticBasis,
    wal: crate::physical_runtime::durability::PhysicalWalAppendPort,
    wal_barrier: crate::physical_runtime::durability::PhysicalWalGroupBarrierPort,
    root_work: crate::physical_runtime::durability::PhysicalRootPublicationWorkPort,
    root_owner: crate::physical_runtime::durability::PhysicalCurrentRootOwner,
    residency: PhysicalResidencyWorkPort,
    mutation: CanonicalRecordMutationPort,
    generation: crate::physical_runtime::LifecycleGeneration,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    residue: RecordPublicationResidueObservation,
    preparation: Mutex<RecordPreparationState>,
    retirement_owner: Mutex<()>,
    retirement_release: Mutex<Option<retirement_release::PendingRetirementRelease>>,
    arena_evacuation: Mutex<Option<arena_evacuation::ArenaEvacuationProgress>>,
    extent_copy: Mutex<Option<extent_copy::ExtentCopySession>>,
    copy_obligation: Mutex<Option<extent_copy::SharedCopyObligation>>,
    mutations: Arc<crate::physical_runtime::PhysicalMutationRuntimeOwner>,
    #[cfg(feature = "certification-test-authority")]
    retirement_intent_gate: retirement::RetirementIntentGate,
    #[cfg(feature = "certification-test-authority")]
    retirement_kill_seam: AtomicU8,
    #[cfg(feature = "certification-test-authority")]
    retirement_kill_arrived: std::sync::Arc<AtomicBool>,
    #[cfg(feature = "certification-test-authority")]
    stop_before_retirement_delete: AtomicBool,
    #[cfg(feature = "certification-test-authority")]
    stop_after_retirement_delete: AtomicBool,
}

pub(in crate::physical_runtime) struct RecordPublicationTerminalState {
    pub(in crate::physical_runtime) residue: RecordPublicationResidueObservation,
    pub(in crate::physical_runtime) mutations:
        crate::physical_runtime::durability::PhysicalMutationTerminalState,
    pub(in crate::physical_runtime) roots: crate::physical_runtime::PhysicalRecoveryRootBasis,
    pub(in crate::physical_runtime) wal_tail: crate::physical_runtime::PhysicalRecoveryWalTail,
    pub(in crate::physical_runtime) wal_observation:
        crate::physical_runtime::PhysicalWalObservation,
    pub(in crate::physical_runtime) performance_witness:
        worth_store_aspect_native::StorePhysicalBoundaryWitness,
}

pub(in crate::physical_runtime) struct RecordPublicationFoundation {
    pub(in crate::physical_runtime) recovered_checkpoint_custody:
        Option<crate::physical_runtime::durability::PreparedRecoveredCheckpointCustody>,
    pub(in crate::physical_runtime) checkpoint_custody_origin:
        crate::physical_runtime::durability::CheckpointCustodyOrigin,
    pub(in crate::physical_runtime) reader_factory: crate::physical_runtime::record_serving::lifecycle::record_lifecycle::RecordReaderLeaseFactory,
    pub(in crate::physical_runtime) read_protection:
        Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
    pub(in crate::physical_runtime) idempotency:
        crate::physical_runtime::durability::PhysicalMutationIdempotencyRuntimeAuthority,
    pub(in crate::physical_runtime) durability:
        crate::physical_runtime::PhysicalDurabilityObservation,
    pub(in crate::physical_runtime) signal_profile:
        crate::physical_runtime::PhysicalSignalProfileIdentity,
    pub(in crate::physical_runtime) security_basis: [u8; 32],
    pub(in crate::physical_runtime) durability_policy_basis:
        crate::physical_runtime::PhysicalWorkSemanticBasis,
    pub(in crate::physical_runtime) wal: crate::physical_runtime::durability::PhysicalWalAppendPort,
    pub(in crate::physical_runtime) wal_barrier:
        crate::physical_runtime::durability::PhysicalWalGroupBarrierPort,
    pub(in crate::physical_runtime) root_work:
        crate::physical_runtime::durability::PhysicalRootPublicationWorkPort,
    pub(in crate::physical_runtime) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime) current_root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime) previous_root: Option<DurablePhysicalRootManifest>,
    pub(in crate::physical_runtime) displaced_artifacts:
        Vec<crate::physical_runtime::durability::DisplacedArtifact>,
    pub(in crate::physical_runtime) unresolved_retirements:
        Vec<crate::physical_runtime::durability::RetirementRecord>,
    pub(in crate::physical_runtime) publication_retention: AdmittedPublicationRetention,
    pub(in crate::physical_runtime) free_space: DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime) allocation_frontier: RecordAllocationFrontier,
    pub(in crate::physical_runtime) residue: RecordPublicationResidueObservation,
    pub(in crate::physical_runtime) frame_ports: RecordFramePorts,
    pub(in crate::physical_runtime) recovery_allocation:
        crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    pub(in crate::physical_runtime) generation: crate::physical_runtime::LifecycleGeneration,
    pub(in crate::physical_runtime) lifecycle:
        Arc<crate::physical_runtime::lifecycle::LifecycleState>,
}

struct RecordPreparationState {
    allocation_frontier: RecordAllocationFrontier,
    arenas: Option<super::super::arena::SharedArenaAllocationOwner>,
    recovered_retiring_arena: Option<worth_store_physical_format::ExtentArenaId>,
    recovered_copy_destination: Option<extent_copy::SharedCopyDestination>,
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<PhysicalStoreWorkRuntime>,
        planning_read: CanonicalRecordReadPort,
        mutation: CanonicalRecordMutationPort,
        foundation: RecordPublicationFoundation,
    ) -> Arc<Self> {
        let writeback = mutation.frame_writeback_port(foundation.frame_ports.clone());
        let displaced = foundation.displaced_artifacts.clone();
        let unresolved_retirements = foundation.unresolved_retirements.clone();
        let bootstrap_frame_ports = foundation.frame_ports.clone();
        let bootstrap_lifecycle = Arc::clone(&foundation.lifecycle);
        let root_owner = crate::physical_runtime::durability::PhysicalCurrentRootOwner::new(
            runtime,
            foundation.checkpoint_custody_origin,
            foundation.recovered_checkpoint_custody,
            foundation.current_root.clone(),
            foundation.previous_root,
            foundation.free_space.clone(),
            foundation.read_protection,
            foundation.publication_retention.into_admission(),
            foundation.recovery_allocation,
        );
        for charge in &displaced {
            root_owner.restore_displaced(charge.source_root, charge.artifact, charge.bytes);
        }
        for record in foundation.unresolved_retirements {
            if displaced
                .iter()
                .any(|charge| charge.artifact == record.artifact)
            {
                continue;
            }
            root_owner.restore_displaced(record.source_root, record.artifact, record.bytes);
        }
        foundation
            .wal
            .bind_publication_admission(root_owner.publication_admission());
        let director = Arc::new_cyclic(|director| Self {
            reader_factory: foundation.reader_factory,
            runtime: Arc::downgrade(runtime),
            mutation_identity: runtime.submission.mutation_submission(),
            idempotency: foundation.idempotency,
            durability: foundation.durability,
            signal_profile: foundation.signal_profile,
            security_basis: foundation.security_basis,
            durability_policy_basis: foundation.durability_policy_basis,
            wal: foundation.wal,
            wal_barrier: foundation.wal_barrier,
            root_work: foundation.root_work,
            root_owner,
            residency: PhysicalResidencyWorkPort::new(
                foundation.frame_ports,
                CanonicalFrameReadSource::new(planning_read),
                writeback,
                foundation.lifecycle,
            ),
            mutation,
            generation: foundation.generation,
            format: foundation.format,
            access: foundation.access,
            residue: foundation.residue,
            preparation: Mutex::new(RecordPreparationState {
                allocation_frontier: foundation.allocation_frontier,
                arenas: None,
                recovered_retiring_arena: None,
                recovered_copy_destination: None,
            }),
            retirement_owner: Mutex::new(()),
            retirement_release: Mutex::new(None),
            arena_evacuation: Mutex::new(None),
            extent_copy: Mutex::new(None),
            copy_obligation: Mutex::new(None),
            mutations: crate::physical_runtime::PhysicalMutationRuntimeOwner::new(director.clone()),
            #[cfg(feature = "certification-test-authority")]
            retirement_intent_gate: retirement::RetirementIntentGate::new(),
            #[cfg(feature = "certification-test-authority")]
            retirement_kill_seam: AtomicU8::new(0),
            #[cfg(feature = "certification-test-authority")]
            retirement_kill_arrived: std::sync::Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "certification-test-authority")]
            stop_before_retirement_delete: AtomicBool::new(false),
            #[cfg(feature = "certification-test-authority")]
            stop_after_retirement_delete: AtomicBool::new(false),
        });
        if director
            .seed_extent_release(&unresolved_retirements)
            .is_err()
        {
            runtime.health.revoke();
        }
        if director
            .seed_extent_copy(
                &displaced,
                runtime.executor.record_serving_media(),
                &bootstrap_frame_ports,
                bootstrap_lifecycle,
            )
            .is_err()
        {
            runtime.health.revoke();
        }
        director
    }

    pub(in crate::physical_runtime) fn submission(
        director: &Arc<Self>,
    ) -> PhysicalRecordSubmission {
        PhysicalRecordSubmission::new(Arc::downgrade(director))
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_submission(
        director: &Arc<Self>,
    ) -> CertificationPhysicalRecordSubmission {
        CertificationPhysicalRecordSubmission::new(Self::submission(director))
    }

    pub(in crate::physical_runtime) fn checkpoint_custody_snapshot(
        &self,
        checkpoint: worth_store_physical_format::PhysicalCheckpointIdentity,
    ) -> Result<
        crate::physical_runtime::durability::SelectedCheckpointCustodySnapshot,
        crate::physical_runtime::durability::CheckpointCustodyDenial,
    > {
        self.root_owner
            .checkpoint_custody_snapshot(checkpoint, self.format.declaration())
    }

    pub(in crate::physical_runtime) fn require_release_certificate_for_attempt(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
    ) -> Result<(), crate::physical_runtime::durability::ReleaseCertificateCapacityDenial> {
        self.root_owner
            .require_release_certificate_for_attempt(attempt)
    }

    pub(in crate::physical_runtime) fn reserve_release_certificate_capacity(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        key: worth_store_physical_format::ReleaseCustodyHeadKeyV1,
        needed_records: u16,
        worst_case_encoded_bytes: u32,
        head_charge: crate::physical_runtime::durability::ReleaseHeadCapacityCharge,
    ) -> Result<
        crate::physical_runtime::durability::ReleaseCertificateCapacityLease,
        crate::physical_runtime::durability::ReleaseCertificateCapacityDenial,
    > {
        self.root_owner.reserve_release_certificate_capacity(
            attempt,
            key,
            needed_records,
            worst_case_encoded_bytes,
            head_charge,
        )
    }

    pub(in crate::physical_runtime) fn promote_blob_terminal(
        &self,
        claim: &mut crate::physical_runtime::durability::PhysicalBlobSessionClaim,
    ) -> Result<(), crate::physical_runtime::durability::PhysicalBlobTerminalAdmissionDenial> {
        self.root_owner.promote_blob_terminal(claim)
    }

    pub(in crate::physical_runtime) fn admit_blob_reclaim(
        &self,
        claim: &mut crate::physical_runtime::durability::PhysicalBlobSessionClaim,
        inspector: crate::physical_runtime::PhysicalProtectedRootObservation,
        displaced: &[crate::physical_runtime::durability::DisplacedArtifact],
        occupied_attempts: &[[u8; 16]],
        recovered_reservations: Vec<
            crate::physical_runtime::durability::PhysicalRecoveredOriginalDropNoDurableEffect,
        >,
    ) -> Result<
        crate::physical_runtime::durability::PhysicalReclaimAttempt,
        crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial,
    > {
        self.root_owner.admit_blob_reclaim(
            claim,
            inspector,
            displaced,
            occupied_attempts,
            recovered_reservations,
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pause_next_root_capture(
        &self,
        stage: crate::physical_runtime::certification::CertificationReadRootCaptureStage,
    ) -> crate::physical_runtime::certification::CertificationReadRootCapturePauseGate {
        self.root_owner.pause_next_capture_for_certification(stage)
    }

    pub(in crate::physical_runtime) fn residue(&self) -> RecordPublicationResidueObservation {
        self.residue
    }

    pub(in crate::physical_runtime) fn mutation_observation(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationObservation {
        self.mutations.observation()
    }

    pub(in crate::physical_runtime) fn persist_mutation_terminal(
        &self,
        terminal: &crate::physical_runtime::PhysicalMutationTerminalFact,
    ) -> Result<(), crate::physical_runtime::durability::PhysicalMutationTerminalizationDenial>
    {
        match terminal {
            crate::physical_runtime::PhysicalMutationTerminalFact::Completed(fact) => {
                self.idempotency.record_completed(Arc::clone(fact))
            }
            crate::physical_runtime::PhysicalMutationTerminalFact::ProvenNoEffect(_) => Ok(()),
            crate::physical_runtime::PhysicalMutationTerminalFact::Indeterminate(fate) => {
                self.idempotency.record_indeterminate(fate.clone())
            }
        }
    }

    pub(in crate::physical_runtime) fn pause_mutation_at(
        &self,
        checkpoint: crate::physical_runtime::durability::PhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::durability::PhysicalMutationPauseGate {
        self.mutations.pause_at(checkpoint)
    }
}
