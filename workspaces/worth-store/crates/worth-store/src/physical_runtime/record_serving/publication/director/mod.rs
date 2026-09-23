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

mod artifact_scope;
#[cfg(feature = "certification-test-authority")]
mod certification_submission;
mod durable_data;
mod durable_preparation;
mod extent_record_rewrite;
mod group_wal_planning;
mod lifecycle;
mod managed_mutation;
mod pre_seal_cancellation;
mod record_append_fingerprint;
mod retirement;
mod rewrite_anchor;
mod rewrite_pages;
mod rewrite_source_liveness;
mod rewrite_span_selection;
mod root_candidate_execution;
mod root_preparation;
mod root_progression;
mod selected_segment_rewrite;
mod submission;
mod wal_data_planning;

pub use artifact_scope::{InlineArtifactRewritePlanDenial, PlannedInlineRewriteArtifact};
#[cfg(feature = "certification-test-authority")]
pub use certification_submission::CertificationPhysicalRecordSubmission;
pub use submission::PhysicalRecordSubmission;

pub(in crate::physical_runtime) struct RecordPublicationDirector {
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
    pub(in crate::physical_runtime) publication_overheads: Vec<u64>,
    pub(in crate::physical_runtime) free_space: DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime) allocation_frontier: RecordAllocationFrontier,
    pub(in crate::physical_runtime) residue: RecordPublicationResidueObservation,
    pub(in crate::physical_runtime) frame_ports: RecordFramePorts,
    pub(in crate::physical_runtime) generation: crate::physical_runtime::LifecycleGeneration,
    pub(in crate::physical_runtime) lifecycle:
        Arc<crate::physical_runtime::lifecycle::LifecycleState>,
}

struct RecordPreparationState {
    allocation_frontier: RecordAllocationFrontier,
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
        let root_owner = crate::physical_runtime::durability::PhysicalCurrentRootOwner::new(
            runtime,
            foundation.current_root.clone(),
            foundation.previous_root,
            foundation.free_space.clone(),
            foundation.read_protection,
        );
        let retained_wal_tail = foundation
            .durability
            .checkpoint_policy()
            .retained_wal_tail_limit()
            .get()
            .get();
        root_owner.install_retention_profile(
            crate::physical_runtime::durability::PhysicalRetentionProfile::store_default()
                .covering_retained_wal_tail(retained_wal_tail),
        );
        // Live segment and extent files are reachable payload, not excess
        // obsolete bytes. Reopen charges unreclaimed WAL, the overhead of each
        // publication whose WAL frame remains, and displaced generations.
        let publications = foundation.wal.reopened_publications();
        let mut retained = foundation
            .publication_overheads
            .iter()
            .rev()
            .take(usize::try_from(publications).unwrap_or(usize::MAX))
            .fold(0_u64, |total, bytes| total.saturating_add(*bytes));
        retained = retained.saturating_add(foundation.wal.observation().reopened_bytes());
        root_owner.reconstruct_retained_bytes(retained);
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
        Arc::new_cyclic(|director| Self {
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
            }),
            retirement_owner: Mutex::new(()),
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
        })
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

    pub(in crate::physical_runtime) fn current_root(&self) -> DurablePhysicalRootManifest {
        self.root_owner.snapshot().0
    }

    pub(in crate::physical_runtime) fn capture_read_root(
        &self,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        ),
        crate::physical_runtime::PhysicalReadProtectionDenial,
    > {
        self.root_owner.capture_read_root()
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
                self.idempotency.record_indeterminate(*fate)
            }
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn charged_growth_bytes(&self) -> u64 {
        self.root_owner.charged_growth_bytes()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_owe_before_maintenance_barrier(&self) {
        self.wal.certification_owe_before_maintenance_barrier();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_stop_before_retirement_delete(&self) {
        self.stop_before_retirement_delete
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_stop_after_retirement_delete(&self) {
        self.stop_after_retirement_delete
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_public_segment_removal_rejected(
        &self,
    ) -> bool {
        let Some(displaced) = self.root_owner.next_displaced() else {
            return false;
        };
        self.root_work
            .certification_public_removal_rejected(displaced.artifact.files()[0])
            .unwrap_or(false)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pending_publication_count(&self) -> usize {
        self.root_owner.pending_publication_count()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn limit_candidate_growth_bytes(
        &self,
        usable_growth_bytes: u64,
    ) {
        let headroom_bytes = 64 * 1024;
        let profile = crate::physical_runtime::durability::PhysicalRetentionProfile::new(
            usable_growth_bytes
                .checked_add(headroom_bytes)
                .expect("usable growth plus headroom fits u64"),
            4_096,
            headroom_bytes,
            8,
        )
        .expect("growth limits withhold nonzero progress headroom");
        self.root_owner.install_retention_profile(profile);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn planning_snapshot(
        &self,
    ) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
        self.root_owner.snapshot()
    }

    pub(in crate::physical_runtime) fn pause_mutation_at(
        &self,
        checkpoint: crate::physical_runtime::durability::PhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::durability::PhysicalMutationPauseGate {
        self.mutations.pause_at(checkpoint)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn fail_next_wal_member_before_effect(&self) {
        self.wal.fail_next_member_before_effect();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pause_mutation_at_for_certification(
        &self,
        checkpoint: crate::physical_runtime::durability::CertificationPhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::durability::CertificationPhysicalMutationPauseGate {
        self.mutations.pause_at(checkpoint)
    }
}
