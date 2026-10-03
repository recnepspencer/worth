use worth_store_physical_format::CheckpointBindingCompactionHeader;

use super::{
    CapturedCheckpointCandidate, CheckpointCandidateCleanup, CreatedCheckpointCandidate,
    PhysicalCheckpointActionFailure,
};
use crate::physical_runtime::work::{
    PhysicalCheckpointCommandPayload, PhysicalCheckpointWorkAction,
};

impl CreatedCheckpointCandidate {
    pub(in crate::physical_runtime) fn finish(
        mut self,
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
        let (mut encoder, record) = match super::command_encoding::begin_bindings(
            self.encoder,
            header,
            self.command_buffer.as_mut(),
        ) {
            Ok(prepared) => prepared,
            Err(failure) => {
                return Err((
                    CheckpointCandidateCleanup::from_capture(
                        self.basis,
                        self.work,
                        self.custody,
                        self.command_buffer,
                    ),
                    failure,
                ))
            }
        };
        let byte_count = record.len() as u64;
        let mut offset = self.offset;
        if let Err(failure) = self.work.execute(
            self.basis.identity(),
            PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
            Some(record),
            0,
        ) {
            return Err((
                CheckpointCandidateCleanup::from_capture(
                    self.basis,
                    self.work,
                    self.custody,
                    self.command_buffer,
                ),
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
            let record = super::command_encoding::binding(
                &mut encoder,
                binding,
                self.command_buffer.as_mut(),
            )?;
            let byte_count = record.len() as u64;
            self.work.execute(
                self.basis.identity(),
                PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
                Some(record),
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
                CheckpointCandidateCleanup::from_capture(
                    self.basis,
                    self.work,
                    self.custody,
                    self.command_buffer,
                ),
                failure,
            ));
        }
        if let Some(certificates) = self
            .custody
            .as_ref()
            .map(|snapshot| snapshot.certificates())
        {
            for index in 0..certificates.len() {
                let record = self
                    .custody
                    .as_ref()
                    .and_then(|snapshot| snapshot.certificate_frame(index))
                    .expect("the sealed certificate inventory admits every index");
                match encoder.include_certificate_record(record.bytes()) {
                    Ok(()) => {}
                    Err(_) => {
                        return Err((
                            CheckpointCandidateCleanup::from_capture(
                                self.basis,
                                self.work,
                                self.custody,
                                self.command_buffer,
                            ),
                            PhysicalCheckpointActionFailure::PreEffect,
                        ));
                    }
                }
                let byte_count = record.bytes().len() as u64;
                if let Err(failure) = self.work.execute(
                    self.basis.identity(),
                    PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
                    Some(PhysicalCheckpointCommandPayload::Certificate(record)),
                    0,
                ) {
                    return Err((
                        CheckpointCandidateCleanup::from_capture(
                            self.basis,
                            self.work,
                            self.custody,
                            self.command_buffer,
                        ),
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
        let (footer, record) =
            match super::command_encoding::footer(encoder, self.command_buffer.as_mut()) {
                Ok(prepared) => prepared,
                Err(failure) => {
                    return Err((
                        CheckpointCandidateCleanup::from_capture(
                            self.basis,
                            self.work,
                            self.custody,
                            self.command_buffer,
                        ),
                        failure,
                    ))
                }
            };
        let byte_count = record.len() as u64;
        if let Err(failure) = self.work.execute(
            self.basis.identity(),
            PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count },
            Some(record),
            0,
        ) {
            return Err((
                CheckpointCandidateCleanup::from_capture(
                    self.basis,
                    self.work,
                    self.custody,
                    self.command_buffer,
                ),
                failure,
            ));
        }
        self.work
            .pause_after(super::super::yieldpoint::PhysicalCheckpointStep::CandidateFooter);
        drop(self.command_buffer);
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
