use crate::{classify_offline_artifact_family, OfflinePhysicalArtifactFamily};
use sha2::{Digest, Sha256};
use worth_store_physical_backend::{OfflineMediaReadDenial, ReadOnlyOfflineMediaCapability};

use super::{
    OfflineInspectionBudget, OfflineInspectionCancellation, OfflineInspectionCounters,
    OfflineInspectionScope, OfflineWalkedFile,
};
use crate::inspection::resume_checkpoint::{
    CheckpointFileObservation, CheckpointSourceIdentity, OfflineInspectionCheckpoint,
    OfflineInspectionCheckpointCodecDenial,
};
use crate::inspection::resume_revalidation::ResumeRevalidation;

mod execution;

#[derive(Debug)]
pub enum OfflineInspectionDenial {
    Media(OfflineMediaReadDenial),
    ReadBudgetExceeded {
        admitted: u64,
        limit: u64,
    },
    OwnedAllocationBudgetExceeded {
        admitted: u64,
        limit: u64,
    },
    CounterOverflow,
    Cancelled,
    DeadlineExceeded {
        elapsed: std::time::Duration,
        limit: std::time::Duration,
    },
    AbsoluteDeadlineReached {
        deadline: std::time::SystemTime,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineInspectionProgress {
    file_index: usize,
    family: OfflinePhysicalArtifactFamily,
    offset: u64,
    bytes_read: u64,
}

impl OfflineInspectionProgress {
    pub const fn file_index(&self) -> usize {
        self.file_index
    }
    pub const fn family(&self) -> OfflinePhysicalArtifactFamily {
        self.family
    }
    pub const fn offset(&self) -> u64 {
        self.offset
    }
    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

pub struct OfflineInspectionSession {
    media: ReadOnlyOfflineMediaCapability,
    scope: OfflineInspectionScope,
    budget: OfflineInspectionBudget,
    file_index: usize,
    offset: u64,
    file_hasher: Sha256,
    read_buffer: Vec<u8>,
    walked: Vec<OfflineWalkedFile>,
    counters: OfflineInspectionCounters,
    cancellation: OfflineInspectionCancellation,
    started_at: std::time::Instant,
    clock: std::sync::Arc<dyn super::OfflineInspectionClock>,
    elapsed_before_resume: std::time::Duration,
    resume_revalidation: Option<ResumeRevalidation>,
    resident_owned_allocation_bytes: u64,
}

impl OfflineInspectionSession {
    pub(crate) fn new(
        media: ReadOnlyOfflineMediaCapability,
        scope: OfflineInspectionScope,
        budget: OfflineInspectionBudget,
        cancellation: OfflineInspectionCancellation,
        started_at: std::time::Instant,
        clock: std::sync::Arc<dyn super::OfflineInspectionClock>,
    ) -> Result<Self, crate::OfflineMediaAcquisitionDenial> {
        let requested_session_owned = media
            .resident_owned_allocation_bytes()
            .checked_add(
                u64::try_from(budget.max_buffer_bytes())
                    .ok()
                    .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?,
            )
            .and_then(|bytes| {
                bytes.checked_add(
                    u64::try_from(media.file_count())
                        .ok()?
                        .checked_mul(std::mem::size_of::<OfflineWalkedFile>() as u64)?,
                )
            })
            .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        let requested_peak = requested_session_owned.max(media.peak_owned_allocation_bytes());
        if requested_peak > budget.maximum_owned_allocation_bytes() {
            return Err(crate::OfflineMediaAcquisitionDenial::BudgetExceeded {
                dimension: crate::OfflineMediaAcquisitionDimension::OwnedAllocationBytes,
                admitted: requested_peak,
                limit: budget.maximum_owned_allocation_bytes(),
            });
        }
        let mut read_buffer = Vec::new();
        read_buffer
            .try_reserve_exact(budget.max_buffer_bytes())
            .map_err(|_| crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        read_buffer.resize(budget.max_buffer_bytes(), 0);
        let mut walked = Vec::new();
        walked
            .try_reserve_exact(media.file_count())
            .map_err(|_| crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        let resident_owned_allocation_bytes = media
            .resident_owned_allocation_bytes()
            .checked_add(
                u64::try_from(read_buffer.capacity())
                    .ok()
                    .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?,
            )
            .and_then(|bytes| {
                bytes.checked_add(
                    u64::try_from(walked.capacity())
                        .ok()?
                        .checked_mul(std::mem::size_of::<OfflineWalkedFile>() as u64)?,
                )
            })
            .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        let peak_owned_allocation_bytes =
            resident_owned_allocation_bytes.max(media.peak_owned_allocation_bytes());
        if peak_owned_allocation_bytes > budget.maximum_owned_allocation_bytes() {
            return Err(crate::OfflineMediaAcquisitionDenial::BudgetExceeded {
                dimension: crate::OfflineMediaAcquisitionDimension::OwnedAllocationBytes,
                admitted: peak_owned_allocation_bytes,
                limit: budget.maximum_owned_allocation_bytes(),
            });
        }
        let counters = OfflineInspectionCounters::with_allocations(
            budget.max_buffer_bytes(),
            peak_owned_allocation_bytes,
        )
        .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        Ok(Self {
            media,
            scope,
            budget,
            file_index: 0,
            offset: 0,
            file_hasher: Sha256::new(),
            read_buffer,
            walked,
            counters,
            cancellation,
            started_at,
            clock,
            elapsed_before_resume: std::time::Duration::ZERO,
            resume_revalidation: None,
            resident_owned_allocation_bytes,
        })
    }

    pub(crate) fn apply_checkpoint(
        &mut self,
        checkpoint: &OfflineInspectionCheckpoint,
    ) -> Result<bool, crate::OfflineMediaAcquisitionDenial> {
        if !self.checkpoint_matches(checkpoint) {
            self.counters = self
                .counters
                .record_checkpoint_rejection()
                .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
            return Ok(false);
        }
        self.media
            .revalidate_consistency()
            .map_err(crate::OfflineMediaAcquisitionDenial::Media)?;
        let revalidation_bytes = ResumeRevalidation::required_owned_allocation_bytes(checkpoint)
            .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        let admitted = self
            .resident_owned_allocation_bytes
            .checked_add(revalidation_bytes)
            .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        if admitted > self.budget.maximum_owned_allocation_bytes() {
            self.counters = self
                .counters
                .record_checkpoint_rejection()
                .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
            return Ok(false);
        }
        self.counters = self.counters.record_owned_allocation_peak(admitted);
        self.resume_revalidation = ResumeRevalidation::from_checkpoint(checkpoint)?;
        Ok(self.resume_revalidation.is_some())
    }

    pub(crate) fn apply_owned_checkpoint(
        &mut self,
        checkpoint: OfflineInspectionCheckpoint,
    ) -> Result<bool, crate::OfflineMediaAcquisitionDenial> {
        if !self.checkpoint_matches(&checkpoint) {
            self.reject_checkpoint()?;
            return Ok(false);
        }
        let revalidation_bytes =
            ResumeRevalidation::required_owned_allocation_bytes(&checkpoint)
                .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        let admitted = self
            .resident_owned_allocation_bytes
            .checked_add(revalidation_bytes)
            .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        if admitted > self.budget.maximum_owned_allocation_bytes() {
            self.reject_checkpoint()?;
            return Ok(false);
        }
        self.media
            .revalidate_consistency()
            .map_err(crate::OfflineMediaAcquisitionDenial::Media)?;
        self.counters = self.counters.record_owned_allocation_peak(admitted);
        self.resume_revalidation = ResumeRevalidation::from_owned_checkpoint(checkpoint);
        Ok(self.resume_revalidation.is_some())
    }

    pub(crate) fn checkpoint_decode_allocation_limit(&self) -> u64 {
        self.budget
            .maximum_owned_allocation_bytes()
            .saturating_sub(self.resident_owned_allocation_bytes)
    }

    pub(crate) fn reject_checkpoint(&mut self) -> Result<(), crate::OfflineMediaAcquisitionDenial> {
        self.counters = self
            .counters
            .record_checkpoint_rejection()
            .ok_or(crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        Ok(())
    }

    fn checkpoint_matches(&self, checkpoint: &OfflineInspectionCheckpoint) -> bool {
        if !self.media.basis().is_content_addressed_closure()
            || checkpoint.basis_identity != self.media.basis().identity()
            || checkpoint.file_index > self.media.file_count()
            || checkpoint.completed.len() != checkpoint.file_index
        {
            return false;
        }
        for observed in &checkpoint.completed {
            let Some(source) = self.media.file(observed.file_index()) else {
                return false;
            };
            let family = classify_offline_artifact_family(
                source
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
            );
            let Some(expected) = self.media.basis().expected_artifact(source.path()) else {
                return false;
            };
            if !self.scope.includes(family)
                || !observed.admits(observed.file_index(), source, family, expected)
            {
                return false;
            }
        }
        match (checkpoint.offset, &checkpoint.partial_source) {
            (0, None) => true,
            (offset, Some(expected)) => self
                .media
                .file(checkpoint.file_index)
                .is_some_and(|source| offset <= source.length() && expected.matches(source)),
            _ => false,
        }
    }

    pub fn checkpoint(
        &self,
    ) -> Result<OfflineInspectionCheckpoint, OfflineInspectionCheckpointCodecDenial> {
        let requested_checkpoint_bytes = u64::try_from(self.walked.len())
            .ok()
            .and_then(|count| {
                count.checked_mul(std::mem::size_of::<CheckpointFileObservation>() as u64)
            })
            .and_then(|bytes| {
                bytes.checked_add(u64::try_from(self.media.basis().identity().len()).ok()?)
            })
            .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?;
        let live_revalidation_bytes = self
            .resume_revalidation
            .as_ref()
            .map_or(0, ResumeRevalidation::owned_allocation_bytes);
        let admitted = self
            .resident_owned_allocation_bytes
            .checked_add(live_revalidation_bytes)
            .and_then(|bytes| bytes.checked_add(requested_checkpoint_bytes))
            .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?;
        if admitted > self.budget.maximum_owned_allocation_bytes() {
            return Err(
                OfflineInspectionCheckpointCodecDenial::OwnedAllocationLimitExceeded {
                    admitted,
                    limit: self.budget.maximum_owned_allocation_bytes(),
                },
            );
        }
        let mut completed = Vec::new();
        completed
            .try_reserve_exact(self.walked.len())
            .map_err(|_| OfflineInspectionCheckpointCodecDenial::AllocationFailed)?;
        let actual_checkpoint_bytes = u64::try_from(completed.capacity())
            .ok()
            .and_then(|count| {
                count.checked_mul(std::mem::size_of::<CheckpointFileObservation>() as u64)
            })
            .and_then(|bytes| {
                bytes.checked_add(u64::try_from(self.media.basis().identity().len()).ok()?)
            })
            .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?;
        let actual_admitted = self
            .resident_owned_allocation_bytes
            .checked_add(live_revalidation_bytes)
            .and_then(|bytes| bytes.checked_add(actual_checkpoint_bytes))
            .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?;
        if actual_admitted > self.budget.maximum_owned_allocation_bytes() {
            return Err(
                OfflineInspectionCheckpointCodecDenial::OwnedAllocationLimitExceeded {
                    admitted: actual_admitted,
                    limit: self.budget.maximum_owned_allocation_bytes(),
                },
            );
        }
        completed.extend(
            self.walked
                .iter()
                .map(|file| CheckpointFileObservation::from_walked(file.source_index(), file)),
        );
        let elapsed = self
            .elapsed_before_resume
            .checked_add(self.started_at.elapsed())
            .ok_or(OfflineInspectionCheckpointCodecDenial::SizeLimitExceeded)?;
        let partial_source = if self.offset > 0 {
            Some(CheckpointSourceIdentity::from_source(
                self.media
                    .file(self.file_index)
                    .ok_or(OfflineInspectionCheckpointCodecDenial::InvalidEncoding)?,
            ))
        } else {
            None
        };
        Ok(OfflineInspectionCheckpoint {
            basis_identity: self.media.basis().identity().to_owned(),
            file_index: self.file_index,
            offset: self.offset,
            counters: self.counters,
            elapsed,
            completed,
            partial_source,
            partial_digest: (self.offset > 0).then(|| self.file_hasher.clone().finalize().into()),
        })
    }
}
