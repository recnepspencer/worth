use worth_store_physical_format::CheckpointBindingCompactionHeader;

use super::{
    CapturedCheckpointCandidate, CheckpointCandidateCleanup, CreatedCheckpointCandidate,
    PhysicalCheckpointActionFailure,
};
use crate::physical_runtime::work::PhysicalCheckpointWorkAction;

impl CreatedCheckpointCandidate {
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
            super::super::yieldpoint::PhysicalCheckpointStep::CandidateBindingCompactionHeader,
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
            self.work.pause_after(
                super::super::yieldpoint::PhysicalCheckpointStep::CandidateBindingRecord,
            );
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
        if let Some(certificates) = self
            .custody
            .as_ref()
            .and_then(|snapshot| snapshot.certificates())
        {
            for certificate in certificates.iter() {
                let record = match encoder
                    .encode_certificate_record(certificate.kind(), certificate.payload())
                {
                    Ok(record) => record,
                    Err(_) => {
                        return Err((
                            CheckpointCandidateCleanup::new(self.basis, self.work),
                            PhysicalCheckpointActionFailure::PreEffect,
                        ));
                    }
                };
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
                self.work.pause_after(
                    super::super::yieldpoint::PhysicalCheckpointStep::CandidateCertificate,
                );
                offset = offset
                    .checked_add(byte_count)
                    .expect("checkpoint certificate bounds fit u64");
            }
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
            .pause_after(super::super::yieldpoint::PhysicalCheckpointStep::CandidateFooter);
        Ok(CapturedCheckpointCandidate {
            basis: self.basis,
            footer,
            encoded_bytes: offset
                .checked_add(byte_count)
                .expect("checkpoint artifact bounds fit u64"),
            dirty_records: self.dirty_records,
            custody: self.custody,
            work: self.work,
        })
    }
}
