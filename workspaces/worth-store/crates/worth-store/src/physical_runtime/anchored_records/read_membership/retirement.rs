//! Consuming retirement of one original verified read membership share.
use super::{AnchoredRecords, ReadMembershipBacking};
use crate::physical_runtime::PhysicalRecordReadRetirementDenial;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

/// Progress is local ownership progress, never a Store-closed certificate.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AnchoredRecordReadRetirementProgress {
    Advanced,
    Complete,
}

#[derive(Debug)]
pub enum AnchoredRecordReadRetirementDenial {
    ForeignServing,
    Owner(PhysicalRecordReadRetirementDenial),
}

enum Stage {
    OriginalShare,
    Frame,
    Allocation,
    SessionProtection,
    ReaderProtection,
    SessionLifecycle,
    ReaderLifecycle,
    Complete,
}

/// No read capability or original-payload extraction escapes this task.
/// Drop remains an explicitly unbudgeted fallback for unfinished retirement.
#[must_use = "unfinished read retirement still owns physical obligations"]
pub struct AnchoredRecordReadRetirement {
    share: Option<Arc<ReadMembershipBacking>>,
    original: Option<ReadMembershipBacking>,
    stage: Stage,
}

impl AnchoredRecordReadRetirement {
    pub(super) fn new(share: Arc<ReadMembershipBacking>) -> Self {
        Self {
            share: Some(share),
            original: None,
            stage: Stage::OriginalShare,
        }
    }

    /// Advance one fixed owner step under the original request. Every denial
    /// preserves this same task, including all previously completed steps.
    /// Another share's obligations remain that share's responsibility.
    pub fn advance(
        &mut self,
        records: &AnchoredRecords<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<AnchoredRecordReadRetirementProgress, AnchoredRecordReadRetirementDenial> {
        use AnchoredRecordReadRetirementProgress as Progress;
        if matches!(self.stage, Stage::Complete) {
            return Ok(Progress::Complete);
        }
        let original = self
            .share
            .as_deref()
            .or(self.original.as_ref())
            .expect("unfinished retirement owns its original read backing");
        if !original.reader.belongs_to(records.runtime)
            || original.reader.protected_root() != original.root
        {
            return Err(AnchoredRecordReadRetirementDenial::ForeignServing);
        }
        request.consult().map_err(|cause| {
            AnchoredRecordReadRetirementDenial::Owner(
                PhysicalRecordReadRetirementDenial::Execution(cause),
            )
        })?;
        if matches!(self.stage, Stage::OriginalShare) {
            let share = self.share.take().expect("original share is consumed once");
            match Arc::into_inner(share) {
                None => self.stage = Stage::Complete,
                Some(original) => {
                    self.original = Some(original);
                    self.stage = Stage::Frame;
                }
            }
        } else {
            self.advance_original(request)
                .map_err(AnchoredRecordReadRetirementDenial::Owner)?;
        }
        Ok(if matches!(self.stage, Stage::Complete) {
            Progress::Complete
        } else {
            Progress::Advanced
        })
    }

    fn advance_original(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), PhysicalRecordReadRetirementDenial> {
        let original = self
            .original
            .as_mut()
            .expect("unique original backing was moved");
        self.stage = match self.stage {
            Stage::Frame => {
                original._session.try_retire_frame(request)?;
                Stage::Allocation
            }
            Stage::Allocation => {
                original._session.try_retire_allocation(request)?;
                Stage::SessionProtection
            }
            Stage::SessionProtection => {
                original._session.try_retire_protection(request)?;
                Stage::ReaderProtection
            }
            Stage::ReaderProtection => {
                original.reader.try_retire_protection(request)?;
                Stage::SessionLifecycle
            }
            Stage::SessionLifecycle => {
                original._session.retire_lifecycle();
                Stage::ReaderLifecycle
            }
            Stage::ReaderLifecycle => {
                original.reader.retire_lifecycle();
                Stage::Complete
            }
            Stage::OriginalShare | Stage::Complete => unreachable!("unique retirement step"),
        };
        Ok(())
    }

    pub fn into_retired(self) -> Result<RetiredAnchoredRecordReadMembership, Self> {
        if matches!(self.stage, Stage::Complete) {
            Ok(RetiredAnchoredRecordReadMembership { _original: self })
        } else {
            Err(self)
        }
    }
}

/// Physical obligations were discharged or this nonlast local share released.
/// Original allocations and admissions remain owned until unbudgeted reclaim.
pub struct RetiredAnchoredRecordReadMembership {
    _original: AnchoredRecordReadRetirement,
}

impl RetiredAnchoredRecordReadMembership {
    /// Explicitly unbudgeted destruction of the original retained allocations.
    pub fn reclaim(self) {
        drop(self);
    }
}
