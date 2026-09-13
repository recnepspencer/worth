use crate::{
    classify_offline_artifact_family, observe_bounded_physical_bytes, OfflinePhysicalArtifactFamily,
};
use sha2::{Digest, Sha256};

use super::{OfflineInspectionDenial, OfflineInspectionProgress, OfflineInspectionSession};
use crate::inspection::resume_revalidation::ResumeRevalidationOutcome;
use crate::inspection::{OfflineWalkedFile, StructurallyWalkedMedia};

impl OfflineInspectionSession {
    pub fn advance(
        &mut self,
    ) -> Result<Option<OfflineInspectionProgress>, OfflineInspectionDenial> {
        loop {
            self.reject_interruption()?;
            let Some(identity) = self.media.file(self.file_index) else {
                return Ok(None);
            };
            let file_length = identity.length();
            let family = classify_offline_artifact_family(
                identity
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
            );
            if !self.scope.includes(family) {
                self.file_index = self
                    .file_index
                    .checked_add(1)
                    .ok_or(OfflineInspectionDenial::CounterOverflow)?;
                self.offset = 0;
                continue;
            }
            if self.offset == file_length {
                self.complete_current_file(family)?;
                continue;
            }
            if self.counters.bytes_read() >= self.budget.max_total_read_bytes() {
                return Err(OfflineInspectionDenial::ReadBudgetExceeded {
                    admitted: self.counters.bytes_read(),
                    limit: self.budget.max_total_read_bytes(),
                });
            }
            let remaining_budget =
                usize::try_from(self.budget.max_total_read_bytes() - self.counters.bytes_read())
                    .unwrap_or(usize::MAX);
            let mut requested = self.budget.max_buffer_bytes().min(remaining_budget);
            if let Some(revalidation) = &self.resume_revalidation {
                requested = revalidation.cap_read_bytes(self.file_index, self.offset, requested);
            }
            let observation = self
                .media
                .read_bounded_into(
                    self.file_index,
                    self.offset,
                    &mut self.read_buffer[..requested],
                )
                .map_err(OfflineInspectionDenial::Media)?;
            if observation.bytes_read() == 0 {
                self.complete_current_file(family)?;
                continue;
            }
            let bounded = observe_bounded_physical_bytes(
                family,
                self.offset,
                &self.read_buffer[..observation.bytes_read()],
            );
            self.file_hasher
                .update(&self.read_buffer[..observation.bytes_read()]);
            let read = bounded.length();
            self.counters = self
                .counters
                .record_read(requested, read as usize, self.offset == 0)
                .ok_or(OfflineInspectionDenial::CounterOverflow)?;
            let progress_file_index = self.file_index;
            let progress_offset = self.offset;
            self.offset = self
                .offset
                .checked_add(read)
                .ok_or(OfflineInspectionDenial::CounterOverflow)?;
            self.validate_partial_checkpoint_prefix()?;
            let progress = OfflineInspectionProgress {
                file_index: progress_file_index,
                family,
                offset: progress_offset,
                bytes_read: read,
            };
            if self.offset == file_length {
                self.complete_current_file(family)?;
            }
            return Ok(Some(progress));
        }
    }

    fn complete_current_file(
        &mut self,
        family: OfflinePhysicalArtifactFamily,
    ) -> Result<(), OfflineInspectionDenial> {
        let source = self
            .media
            .file(self.file_index)
            .ok_or(OfflineInspectionDenial::Media(
                worth_store_physical_backend::OfflineMediaReadDenial::InvalidFileIndex,
            ))?;
        let identity_owned_bytes = source
            .owned_allocation_bytes()
            .ok_or(OfflineInspectionDenial::CounterOverflow)?;
        let admitted_owned_bytes = self
            .resident_owned_allocation_bytes
            .checked_add(identity_owned_bytes)
            .ok_or(OfflineInspectionDenial::CounterOverflow)?;
        if admitted_owned_bytes > self.budget.maximum_owned_allocation_bytes() {
            return Err(OfflineInspectionDenial::OwnedAllocationBudgetExceeded {
                admitted: admitted_owned_bytes,
                limit: self.budget.maximum_owned_allocation_bytes(),
            });
        }
        self.resident_owned_allocation_bytes = admitted_owned_bytes;
        self.counters = self
            .counters
            .record_owned_allocation_peak(admitted_owned_bytes);
        let identity = source.clone();
        let digest: [u8; 32] = self.file_hasher.clone().finalize().into();
        if let Some(revalidation) = &self.resume_revalidation {
            let outcome =
                revalidation.validate_completed_file(self.file_index, identity.length(), digest);
            self.record_revalidation_outcome(outcome, true)?;
        }
        self.walked.push(OfflineWalkedFile::new(
            self.file_index,
            identity,
            family,
            digest,
        ));
        self.file_index = self
            .file_index
            .checked_add(1)
            .ok_or(OfflineInspectionDenial::CounterOverflow)?;
        self.offset = 0;
        self.file_hasher = Sha256::new();
        Ok(())
    }

    fn validate_partial_checkpoint_prefix(&mut self) -> Result<(), OfflineInspectionDenial> {
        let outcome = self.resume_revalidation.as_ref().and_then(|revalidation| {
            revalidation.validate_partial_prefix(
                self.file_index,
                self.offset,
                self.file_hasher.clone().finalize().into(),
            )
        });
        if let Some(outcome) = outcome {
            self.record_revalidation_outcome(outcome, false)?;
        }
        Ok(())
    }

    fn record_revalidation_outcome(
        &mut self,
        outcome: ResumeRevalidationOutcome,
        completed_file: bool,
    ) -> Result<(), OfflineInspectionDenial> {
        match outcome {
            ResumeRevalidationOutcome::Continue { bytes } => {
                self.counters = self
                    .counters
                    .record_checkpoint_revalidation(u64::from(completed_file), bytes)
                    .ok_or(OfflineInspectionDenial::CounterOverflow)?;
            }
            ResumeRevalidationOutcome::Completed { bytes } => {
                self.counters = self
                    .counters
                    .record_checkpoint_revalidation(u64::from(completed_file), bytes)
                    .ok_or(OfflineInspectionDenial::CounterOverflow)?;
                self.resume_revalidation = None;
            }
            ResumeRevalidationOutcome::Rejected => {
                self.counters = self
                    .counters
                    .record_checkpoint_rejection()
                    .ok_or(OfflineInspectionDenial::CounterOverflow)?;
                self.resume_revalidation = None;
            }
        }
        Ok(())
    }

    fn reject_interruption(&self) -> Result<(), OfflineInspectionDenial> {
        if self.cancellation.is_cancelled() {
            return Err(OfflineInspectionDenial::Cancelled);
        }
        if let Some(deadline) = self.budget.deadline() {
            if self.clock.now() >= deadline {
                return Err(OfflineInspectionDenial::AbsoluteDeadlineReached { deadline });
            }
        }
        if let Some(limit) = self.budget.max_elapsed() {
            let elapsed = self
                .elapsed_before_resume
                .saturating_add(self.started_at.elapsed());
            if elapsed >= limit {
                return Err(OfflineInspectionDenial::DeadlineExceeded { elapsed, limit });
            }
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<StructurallyWalkedMedia, OfflineInspectionDenial> {
        while self.advance()?.is_some() {}
        self.media
            .revalidate_consistency()
            .map_err(OfflineInspectionDenial::Media)?;
        self.media
            .validate_content_closure(
                self.walked
                    .iter()
                    .map(|file| (file.path(), file.length(), file.content_digest())),
            )
            .map_err(OfflineInspectionDenial::Media)?;
        let consistency_basis = self.media.into_consistency_basis();
        Ok(StructurallyWalkedMedia::new(
            consistency_basis,
            self.walked,
            self.counters,
        ))
    }
}
