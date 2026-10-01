use std::sync::Arc;

use crate::physical_runtime::durability::{
    PhysicalCheckpointCaptureFoundation, PhysicalCheckpointRuntimeOwner,
    PhysicalCheckpointWorkPort, PhysicalRootPublicationWorkPort, PhysicalWalAppendPort,
    PhysicalWalGroupBarrierPort, PhysicalWalReclamationFoundation, PhysicalWalReclamationOwner,
};
use crate::physical_runtime::record_serving::{
    CanonicalRecordMutationPort, CanonicalRecordReadPort, RecordAllocationFrontier,
    RecordPublicationDirector, RecordPublicationFoundation, RecordServingState,
};
use crate::physical_runtime::{LifecycleGeneration, PhysicalSignalProfileIdentity};

use super::super::durability_bootstrap::ReopenedPhysicalDurabilityOwners;
use super::work_runtime::InstalledPhysicalWorkRuntime;

pub(super) struct PhysicalRecordServingAssembly {
    state: RecordServingState,
    recovered_checkpoint_custody:
        Option<crate::physical_runtime::durability::PreparedRecoveredCheckpointCustody>,
    checkpoint_custody_origin: crate::physical_runtime::durability::CheckpointCustodyOrigin,
    allocation: RecordAllocationFrontier,
    frame_ports: crate::physical_runtime::record_serving::RecordFramePorts,
    recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    generation: LifecycleGeneration,
    signal_profile: PhysicalSignalProfileIdentity,
    lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
}

pub(super) struct InstalledPhysicalRecordServing {
    pub(super) format: crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat,
    pub(super) access: crate::physical_runtime::record_serving::AdmittedRecordAccessPolicy,
    pub(super) publication: Arc<RecordPublicationDirector>,
    pub(super) checkpoint: Arc<PhysicalCheckpointRuntimeOwner>,
    pub(super) root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounters,
}

impl PhysicalRecordServingAssembly {
    pub(super) fn new(
        state: RecordServingState,
        checkpoint_custody_origin: crate::physical_runtime::durability::CheckpointCustodyOrigin,
        recovered_checkpoint_custody: Option<
            crate::physical_runtime::durability::PreparedRecoveredCheckpointCustody,
        >,
        allocation: RecordAllocationFrontier,
        frame_ports: crate::physical_runtime::record_serving::RecordFramePorts,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        generation: LifecycleGeneration,
        signal_profile: PhysicalSignalProfileIdentity,
        lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    ) -> Self {
        Self {
            state,
            recovered_checkpoint_custody,
            checkpoint_custody_origin,
            allocation,
            frame_ports,
            recovery_allocation,
            generation,
            signal_profile,
            lifecycle,
        }
    }

    pub(super) fn install(
        self,
        work: &InstalledPhysicalWorkRuntime,
        durability: &ReopenedPhysicalDurabilityOwners,
        record_owner: &crate::physical_runtime::record_serving::RecordServingOwner,
        read_protection: Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
        publication_retention: crate::physical_runtime::record_serving::AdmittedPublicationRetention,
    ) -> InstalledPhysicalRecordServing {
        let read = CanonicalRecordReadPort::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
        );
        let mutation = CanonicalRecordMutationPort::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
        );
        let wal = PhysicalWalAppendPort::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
            durability.wal.clone(),
            durability.durability.grouping_authority(),
            durability.durability.idempotency_authority(),
            durability.durability.observation(),
        );
        let wal_barrier = PhysicalWalGroupBarrierPort::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
            durability.durability.observation(),
            durability.wal.clone(),
        );
        let checkpoint_work = PhysicalCheckpointWorkPort::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
        );
        let root_work = PhysicalRootPublicationWorkPort::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
        );
        let reclamation = PhysicalWalReclamationOwner::new(PhysicalWalReclamationFoundation::new(
            &work.runtime,
            self.generation,
            work.admission,
            work.scheduler.clone(),
            Arc::clone(&work.record_work),
            durability.wal.clone(),
        ));
        let publication = RecordPublicationDirector::new(
            &work.runtime,
            read,
            mutation,
            RecordPublicationFoundation {
                recovered_checkpoint_custody: self.recovered_checkpoint_custody,
                checkpoint_custody_origin: self.checkpoint_custody_origin,
                reader_factory: record_owner.reader_factory(),
                read_protection,
                idempotency: durability.durability.idempotency_authority(),
                durability: durability.durability.observation(),
                signal_profile: self.signal_profile,
                security_basis: work
                    .record_work
                    .security()
                    .receipt()
                    .identity()
                    .stable_fingerprint(),
                durability_policy_basis: work.record_work.durability_policy_basis(),
                wal: wal.clone(),
                wal_barrier,
                root_work,
                format: self.state.format,
                access: self.state.access,
                current_root: self.state.current_root,
                previous_root: self.state.previous_root,
                displaced_artifacts: self.state.displaced_artifacts,
                unresolved_retirements: durability.unresolved_retirements.clone(),
                publication_retention,
                free_space: self.state.free_space,
                allocation_frontier: self.allocation,
                residue: self.state.publication_residue,
                frame_ports: self.frame_ports.clone(),
                recovery_allocation: self.recovery_allocation,
                generation: self.generation,
                lifecycle: self.lifecycle,
            },
        );
        let checkpoint = PhysicalCheckpointRuntimeOwner::new(
            PhysicalCheckpointCaptureFoundation {
                publication: Arc::clone(&publication),
                wal,
                binding_compaction: durability.durability.binding_compaction_authority(),
                frames: self.frame_ports,
                work: checkpoint_work,
                durability: durability.durability.observation(),
                reclamation,
                selected_checkpoint_sequence: durability.selected_checkpoint_sequence,
            },
            &work.runtime,
        );
        InstalledPhysicalRecordServing {
            format: self.state.format,
            access: self.state.access,
            publication,
            checkpoint,
            root_protocol_counters: self.state.root_protocol_counters,
        }
    }
}
