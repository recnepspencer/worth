use std::sync::Arc;

use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointDirtyFrameBasis, CheckpointStreamEncoder,
    CheckpointStreamFooter,
};

use super::capture::PhysicalCheckpointCaptureBasis;
use super::{PhysicalCheckpointActionFailure, PhysicalCheckpointWorkPort};
use crate::physical_runtime::work::{
    CompletedPhysicalCheckpointAction, PhysicalCheckpointWorkAction,
};

pub(in crate::physical_runtime) struct CreatedCheckpointCandidate {
    basis: PhysicalCheckpointCaptureBasis,
    encoder: CheckpointStreamEncoder,
    offset: u64,
    dirty_records: u64,
    work: PhysicalCheckpointWorkPort,
}

pub(in crate::physical_runtime) struct CapturedCheckpointCandidate {
    basis: PhysicalCheckpointCaptureBasis,
    footer: CheckpointStreamFooter,
    encoded_bytes: u64,
    dirty_records: u64,
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
    ) -> Result<Self, (CheckpointCandidateCleanup, PhysicalCheckpointActionFailure)> {
        let (encoder, header) = CheckpointStreamEncoder::begin(basis.source());
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

    pub(in crate::physical_runtime) fn finish(
        self,
        binding_compaction: &crate::physical_runtime::durability::PhysicalMutationBindingCompactionCutover<'_>,
    ) -> Result<
        CapturedCheckpointCandidate,
        (CheckpointCandidateCleanup, PhysicalCheckpointActionFailure),
    > {
        let header = CheckpointBindingCompactionHeader::new(
            binding_compaction.generation().get(),
            binding_compaction.wal_cutoff_lsn_exclusive(),
        )
        .expect("a prospective compaction has a nonzero generation and WAL cutoff");
        let (mut encoder, record) = self.encoder.begin_binding_compaction(header);
        let byte_count = record.len() as u64;
        let mut offset = self.offset;
        if let Err(failure) = self.work.execute(
            self.basis.identity(),
            PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
            Some(record.into_boxed_slice()),
            0,
        ) {
            return Err((
                CheckpointCandidateCleanup::new(self.basis, self.work),
                failure,
            ));
        }
        self.work.pause_after(
            super::yieldpoint::PhysicalCheckpointStep::CandidateBindingCompactionHeader,
        );
        offset = offset
            .checked_add(byte_count)
            .expect("checkpoint artifact bounds fit u64");
        let stream_result = binding_compaction.for_each_record(|binding| {
            let record = encoder
                .encode_binding_record(binding)
                .expect("Store compaction construction admitted every bounded record");
            let byte_count = record.len() as u64;
            self.work.execute(
                self.basis.identity(),
                PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
                Some(record.into_boxed_slice()),
                0,
            )?;
            self.work
                .pause_after(super::yieldpoint::PhysicalCheckpointStep::CandidateBindingRecord);
            offset = offset
                .checked_add(byte_count)
                .expect("checkpoint artifact bounds fit u64");
            Ok(())
        });
        if let Err(failure) = stream_result {
            return Err((
                CheckpointCandidateCleanup::new(self.basis, self.work),
                failure,
            ));
        }
        let (footer, record) = encoder.finish();
        let byte_count = record.len() as u64;
        if let Err(failure) = self.work.execute(
            self.basis.identity(),
            PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
            Some(record.into_boxed_slice()),
            0,
        ) {
            return Err((
                CheckpointCandidateCleanup::new(self.basis, self.work),
                failure,
            ));
        }
        self.work
            .pause_after(super::yieldpoint::PhysicalCheckpointStep::CandidateFooter);
        Ok(CapturedCheckpointCandidate {
            basis: self.basis,
            footer,
            encoded_bytes: offset
                .checked_add(byte_count)
                .expect("checkpoint artifact bounds fit u64"),
            dirty_records: self.dirty_records,
            work: self.work,
        })
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
