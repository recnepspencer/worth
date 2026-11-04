use super::resume_checkpoint::{CheckpointFileObservation, OfflineInspectionCheckpoint};

pub(super) struct ResumeRevalidation {
    completed: Vec<CheckpointFileObservation>,
    target_file_index: usize,
    target_offset: u64,
    partial_digest: Option<[u8; 32]>,
}

pub(super) enum ResumeRevalidationOutcome {
    Continue { bytes: u64 },
    Completed { bytes: u64 },
    Rejected,
}

impl ResumeRevalidation {
    pub(super) fn owned_allocation_bytes(&self) -> u64 {
        u64::try_from(self.completed.capacity())
            .unwrap_or(u64::MAX)
            .saturating_mul(std::mem::size_of::<CheckpointFileObservation>() as u64)
    }
    pub(super) fn required_owned_allocation_bytes(
        checkpoint: &OfflineInspectionCheckpoint,
    ) -> Option<u64> {
        u64::try_from(checkpoint.completed.len())
            .ok()?
            .checked_mul(std::mem::size_of::<CheckpointFileObservation>() as u64)
    }
    pub(super) fn from_checkpoint(
        checkpoint: &OfflineInspectionCheckpoint,
    ) -> Result<Option<Self>, crate::OfflineMediaAcquisitionDenial> {
        if checkpoint.file_index == 0 && checkpoint.offset == 0 {
            return Ok(None);
        }
        let mut completed = Vec::new();
        completed
            .try_reserve_exact(checkpoint.completed.len())
            .map_err(|_| crate::OfflineMediaAcquisitionDenial::SessionAllocationFailed)?;
        completed.extend(checkpoint.completed.iter().cloned());
        Ok(Some(Self {
            completed,
            target_file_index: checkpoint.file_index,
            target_offset: checkpoint.offset,
            partial_digest: match checkpoint.partial_digest {
                Some(digest) => Some(*digest.bytes()),
                None => None,
            },
        }))
    }

    pub(super) fn from_owned_checkpoint(checkpoint: OfflineInspectionCheckpoint) -> Option<Self> {
        if checkpoint.file_index == 0 && checkpoint.offset == 0 {
            return None;
        }
        Some(Self {
            completed: checkpoint.completed,
            target_file_index: checkpoint.file_index,
            target_offset: checkpoint.offset,
            partial_digest: match checkpoint.partial_digest {
                Some(digest) => Some(*digest.bytes()),
                None => None,
            },
        })
    }

    pub(super) fn cap_read_bytes(&self, file_index: usize, offset: u64, requested: usize) -> usize {
        if file_index != self.target_file_index || self.target_offset <= offset {
            return requested;
        }
        let until_boundary = usize::try_from(self.target_offset - offset).unwrap_or(usize::MAX);
        requested.min(until_boundary)
    }

    pub(super) fn validate_completed_file(
        &self,
        file_index: usize,
        bytes: u64,
        digest: [u8; 32],
    ) -> ResumeRevalidationOutcome {
        let Some(expected) = self.completed.get(file_index) else {
            return ResumeRevalidationOutcome::Rejected;
        };
        if expected.file_index() != file_index || expected.content_digest() != digest {
            return ResumeRevalidationOutcome::Rejected;
        }
        if file_index + 1 == self.target_file_index && self.target_offset == 0 {
            ResumeRevalidationOutcome::Completed { bytes }
        } else {
            ResumeRevalidationOutcome::Continue { bytes }
        }
    }

    pub(super) fn validate_partial_prefix(
        &self,
        file_index: usize,
        offset: u64,
        digest: [u8; 32],
    ) -> Option<ResumeRevalidationOutcome> {
        if file_index != self.target_file_index || offset != self.target_offset {
            return None;
        }
        Some(if self.partial_digest == Some(digest) {
            ResumeRevalidationOutcome::Completed { bytes: offset }
        } else {
            ResumeRevalidationOutcome::Rejected
        })
    }
}
