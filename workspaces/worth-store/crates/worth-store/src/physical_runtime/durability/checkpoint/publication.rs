use std::sync::Arc;

use worth_store_physical_format::{
    CheckpointDirtyFrameBasis, CheckpointStreamEncoder, CheckpointStreamFooter,
};

use super::capture::PhysicalCheckpointCaptureBasis;
use super::{PhysicalCheckpointActionFailure, PhysicalCheckpointWorkPort};
use crate::physical_runtime::durability::SelectedCheckpointCustodySnapshot;
use crate::physical_runtime::work::{
    CompletedPhysicalCheckpointAction, PhysicalCheckpointWorkAction,
};

#[path = "publication/finish.rs"]
mod finish;

pub(in crate::physical_runtime) struct CreatedCheckpointCandidate {
    basis: PhysicalCheckpointCaptureBasis,
    encoder: CheckpointStreamEncoder,
    offset: u64,
    dirty_records: u64,
    custody: Option<SelectedCheckpointCustodySnapshot>,
    work: PhysicalCheckpointWorkPort,
}

pub(in crate::physical_runtime) struct CapturedCheckpointCandidate {
    basis: PhysicalCheckpointCaptureBasis,
    footer: CheckpointStreamFooter,
    encoded_bytes: u64,
    dirty_records: u64,
    custody: Option<SelectedCheckpointCustodySnapshot>,
    work: PhysicalCheckpointWorkPort,
}

pub(in crate::physical_runtime) struct DurableCheckpointCandidate(CapturedCheckpointCandidate);
pub(in crate::physical_runtime) struct ReplacedCheckpointCandidate(DurableCheckpointCandidate);

pub(in crate::physical_runtime) struct CheckpointCandidateCleanup {
    basis: PhysicalCheckpointCaptureBasis,
    work: PhysicalCheckpointWorkPort,
}

pub(in crate::physical_runtime) struct NamespaceDurableCheckpointPublication {
    basis: PhysicalCheckpointCaptureBasis,
    footer: CheckpointStreamFooter,
    encoded_bytes: u64,
    dirty_records: u64,
    custody: Option<SelectedCheckpointCustodySnapshot>,
    retained_wal_tail: Arc<super::ContiguousRetainedWalTail>,
    binding_compaction: crate::physical_runtime::PhysicalMutationBindingCompaction,
    namespace_sync: CompletedPhysicalCheckpointAction,
}

pub(in crate::physical_runtime) struct PhysicalCheckpointPublication {
    namespace: NamespaceDurableCheckpointPublication,
    wal_reclamation: crate::physical_runtime::PhysicalWalReclamationObservation,
}

pub(super) enum PhysicalCheckpointNamespaceFinalizationFailure {
    Action(PhysicalCheckpointActionFailure),
    BindingCompaction,
}

impl CreatedCheckpointCandidate {
    pub(in crate::physical_runtime) const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.basis
    }

    pub(in crate::physical_runtime) const fn encoded_bytes(&self) -> u64 {
        self.offset
    }

    pub(in crate::physical_runtime) const fn dirty_records(&self) -> u64 {
        self.dirty_records
    }

    pub(in crate::physical_runtime) fn create(
        basis: PhysicalCheckpointCaptureBasis,
        work: PhysicalCheckpointWorkPort,
        custody: Option<SelectedCheckpointCustodySnapshot>,
    ) -> Result<Self, (CheckpointCandidateCleanup, PhysicalCheckpointActionFailure)> {
        if custody.as_ref().is_some_and(|snapshot| {
            snapshot.checkpoint() != basis.identity()
                || snapshot.root().generation() != basis.source().root().generation()
                || snapshot.root().tree_identity() != basis.source().root().tree_identity()
        }) {
            return Err((
                CheckpointCandidateCleanup::new(basis, work),
                PhysicalCheckpointActionFailure::PreEffect,
            ));
        }
        let (encoder, header) = if custody
            .as_ref()
            .and_then(|snapshot| snapshot.certificates())
            .is_some()
        {
            CheckpointStreamEncoder::begin_certified(basis.source())
        } else {
            CheckpointStreamEncoder::begin(basis.source())
        };
        let byte_count = header.len() as u64;
        if let Err(failure) = work.execute(
            basis.identity(),
            PhysicalCheckpointWorkAction::CreateCandidate { byte_count },
            Some(header.into_boxed_slice()),
            0,
        ) {
            return Err((CheckpointCandidateCleanup::new(basis, work), failure));
        }
        Ok(Self {
            basis,
            encoder,
            offset: byte_count,
            dirty_records: 0,
            custody,
            work,
        })
    }

    pub(in crate::physical_runtime) fn append_dirty(
        &mut self,
        basis: CheckpointDirtyFrameBasis,
    ) -> Result<(), PhysicalCheckpointActionFailure> {
        let record = self.encoder.encode_dirty_basis(basis);
        let byte_count = record.len() as u64;
        self.work.execute(
            self.basis.identity(),
            PhysicalCheckpointWorkAction::AppendCandidate {
                offset: self.offset,
                byte_count,
            },
            Some(record.into_boxed_slice()),
            0,
        )?;
        self.work
            .pause_after(super::yieldpoint::PhysicalCheckpointStep::CandidateAppend);
        self.offset = self
            .offset
            .checked_add(byte_count)
            .expect("checkpoint memory and artifact bounds fit u64");
        self.dirty_records = self
            .dirty_records
            .checked_add(1)
            .expect("checkpoint record count fits u64");
        Ok(())
    }

    pub(in crate::physical_runtime) fn remove(
        self,
    ) -> Result<CompletedPhysicalCheckpointAction, PhysicalCheckpointActionFailure> {
        CheckpointCandidateCleanup::new(self.basis, self.work).remove()
    }
}

impl CapturedCheckpointCandidate {
    pub(in crate::physical_runtime) const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.basis
    }

    pub(in crate::physical_runtime) fn synchronize(
        self,
    ) -> Result<
        DurableCheckpointCandidate,
        (CheckpointCandidateCleanup, PhysicalCheckpointActionFailure),
    > {
        if let Err(failure) = self.work.execute(
            self.basis.identity(),
            PhysicalCheckpointWorkAction::SynchronizeCandidate,
            None,
            0,
        ) {
            return Err((
                CheckpointCandidateCleanup::new(self.basis, self.work),
                failure,
            ));
        }
        Ok(DurableCheckpointCandidate(self))
    }

    pub(in crate::physical_runtime) fn remove(
        self,
    ) -> Result<CompletedPhysicalCheckpointAction, PhysicalCheckpointActionFailure> {
        CheckpointCandidateCleanup::new(self.basis, self.work).remove()
    }
}

impl DurableCheckpointCandidate {
    pub(in crate::physical_runtime) const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.0.basis
    }

    pub(in crate::physical_runtime) fn publish(
        self,
    ) -> Result<ReplacedCheckpointCandidate, (Self, PhysicalCheckpointActionFailure)> {
        if let Err(failure) = self.0.work.execute(
            self.0.basis.identity(),
            PhysicalCheckpointWorkAction::PublishCandidate,
            None,
            0,
        ) {
            return Err((self, failure));
        }
        Ok(ReplacedCheckpointCandidate(self))
    }

    pub(in crate::physical_runtime) fn remove(
        self,
    ) -> Result<CompletedPhysicalCheckpointAction, PhysicalCheckpointActionFailure> {
        CheckpointCandidateCleanup::new(self.0.basis, self.0.work).remove()
    }
}

impl CheckpointCandidateCleanup {
    const fn new(basis: PhysicalCheckpointCaptureBasis, work: PhysicalCheckpointWorkPort) -> Self {
        Self { basis, work }
    }

    pub(in crate::physical_runtime) const fn identity(
        &self,
    ) -> worth_store_physical_format::PhysicalCheckpointIdentity {
        self.basis.identity()
    }

    pub(in crate::physical_runtime) fn remove(
        self,
    ) -> Result<CompletedPhysicalCheckpointAction, PhysicalCheckpointActionFailure> {
        remove_candidate(self.basis, &self.work)
    }
}

impl ReplacedCheckpointCandidate {
    pub(in crate::physical_runtime) fn synchronize_namespace(
        self,
        retained_wal_tail: Arc<super::ContiguousRetainedWalTail>,
        binding_cutover: crate::physical_runtime::durability::PhysicalMutationBindingCompactionCutover<'_>,
    ) -> Result<NamespaceDurableCheckpointPublication, PhysicalCheckpointNamespaceFinalizationFailure>
    {
        let candidate = self.0 .0;
        let namespace_sync = candidate
            .work
            .execute(
                candidate.basis.identity(),
                PhysicalCheckpointWorkAction::SynchronizeNamespace,
                None,
                0,
            )
            .map_err(PhysicalCheckpointNamespaceFinalizationFailure::Action)?;
        let binding_compaction = binding_cutover
            .commit_namespace_durable(&namespace_sync)
            .map_err(|_| PhysicalCheckpointNamespaceFinalizationFailure::BindingCompaction)?;
        Ok(NamespaceDurableCheckpointPublication {
            basis: candidate.basis,
            footer: candidate.footer,
            encoded_bytes: candidate.encoded_bytes,
            dirty_records: candidate.dirty_records,
            custody: candidate.custody,
            retained_wal_tail,
            binding_compaction,
            namespace_sync,
        })
    }
}

impl NamespaceDurableCheckpointPublication {
    pub(in crate::physical_runtime) const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.basis
    }

    pub(in crate::physical_runtime) fn custody(
        &self,
    ) -> Option<&SelectedCheckpointCustodySnapshot> {
        self.custody.as_ref()
    }

    pub(in crate::physical_runtime) const fn namespace_sync(
        &self,
    ) -> &CompletedPhysicalCheckpointAction {
        &self.namespace_sync
    }

    pub(in crate::physical_runtime) fn retained_wal_tail(
        &self,
    ) -> &super::ContiguousRetainedWalTail {
        &self.retained_wal_tail
    }

    pub(in crate::physical_runtime) const fn binding_compaction(
        &self,
    ) -> &crate::physical_runtime::PhysicalMutationBindingCompaction {
        &self.binding_compaction
    }

    pub(in crate::physical_runtime) fn with_wal_reclamation(
        self,
        wal_reclamation: crate::physical_runtime::PhysicalWalReclamationObservation,
    ) -> PhysicalCheckpointPublication {
        PhysicalCheckpointPublication {
            namespace: self,
            wal_reclamation,
        }
    }
}

impl PhysicalCheckpointPublication {
    pub(in crate::physical_runtime) const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.namespace.basis()
    }

    pub(in crate::physical_runtime) const fn namespace_sync(
        &self,
    ) -> &CompletedPhysicalCheckpointAction {
        self.namespace.namespace_sync()
    }

    pub(super) fn completed_observation(&self) -> super::CompletedPhysicalCheckpoint {
        super::CompletedPhysicalCheckpoint::new(
            self.namespace.basis,
            self.namespace.footer,
            self.namespace.encoded_bytes,
            self.namespace.dirty_records,
            Arc::clone(&self.namespace.retained_wal_tail),
            self.namespace.binding_compaction.clone(),
            self.wal_reclamation,
        )
    }
}

fn remove_candidate(
    basis: PhysicalCheckpointCaptureBasis,
    work: &PhysicalCheckpointWorkPort,
) -> Result<CompletedPhysicalCheckpointAction, PhysicalCheckpointActionFailure> {
    work.execute(
        basis.identity(),
        PhysicalCheckpointWorkAction::RemoveCandidate,
        None,
        0,
    )
}
