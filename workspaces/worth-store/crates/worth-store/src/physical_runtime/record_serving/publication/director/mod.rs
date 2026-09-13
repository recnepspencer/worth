use std::sync::{Arc, Mutex, Weak};

use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

use crate::physical_runtime::instance::PhysicalStoreWorkRuntime;

use super::super::{
    residency::{frame_loading::CanonicalFrameReadSource, PhysicalResidencyWorkPort},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, CanonicalRecordMutationPort,
    CanonicalRecordReadPort, RecordAllocationFrontier, RecordFramePorts,
    RecordPublicationResidueObservation,
};

#[cfg(feature = "certification-test-authority")]
mod certification_submission;
mod durable_data;
mod durable_preparation;
mod group_wal_planning;
mod lifecycle;
mod managed_mutation;
mod pre_seal_cancellation;
mod root_candidate_execution;
mod root_preparation;
mod root_progression;
mod submission;
mod wal_data_planning;

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
    mutations: Arc<crate::physical_runtime::PhysicalMutationRuntimeOwner>,
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
            root_owner: crate::physical_runtime::durability::PhysicalCurrentRootOwner::new(
                runtime,
                foundation.current_root.clone(),
                foundation.previous_root,
                foundation.free_space.clone(),
            ),
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
            mutations: crate::physical_runtime::PhysicalMutationRuntimeOwner::new(director.clone()),
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
    pub(in crate::physical_runtime) fn pause_mutation_at_for_certification(
        &self,
        checkpoint: crate::physical_runtime::durability::CertificationPhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::durability::CertificationPhysicalMutationPauseGate {
        self.mutations.pause_at(checkpoint)
    }
}
