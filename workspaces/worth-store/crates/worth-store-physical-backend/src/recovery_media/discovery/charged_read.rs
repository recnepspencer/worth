//! The one engine for reads of whole artifacts and exact ranges. Each refusal
//! is decided here, with the read's real length: from the artifact's metadata
//! for a whole read, from the range itself for a range. Past the ceiling is
//! damage; past the caller's grant is that budget's; past the observation's
//! own reads or bytes is the observation's.

use worth_foundational::LimitDimension;
use worth_proof::{DenialTransitionOutcome, TransitionOutcome};

use crate::filesystem_media::{ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeMedia};

use super::super::grant::ReadGrant;
use super::super::refusal::{ArtifactDamage, ReadRefusal};
use super::{
    DiscoveryMediaBacking, FilesystemObservation, ObservedRecoveryArtifact,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};

/// Where a charged read stopped.
pub(super) enum ReadStop<D: LimitDimension, F> {
    Refused(ReadRefusal<D>),
    Failed(F),
}

/// What the tree's read met: its own failure, or one the caller's buffer
/// already classified.
pub(super) enum TreeReadFailure<F> {
    Media(ArtifactTreeFailure),
    Caller(F),
}

impl<D: LimitDimension, F: From<ArtifactDamage>> ReadStop<D, F> {
    pub(super) fn damage(damage: ArtifactDamage) -> Self {
        Self::Failed(F::from(damage))
    }

    /// The observation's own refusal: a limit of its bounds, or a count past
    /// every count.
    pub(super) fn stop(failure: RecoveryDiscoveryFailure) -> Self {
        match failure {
            RecoveryDiscoveryFailure::Limit(limit) => {
                Self::Refused(ReadRefusal::Observation(limit))
            }
            RecoveryDiscoveryFailure::Damage(damage) => Self::damage(damage),
        }
    }
}

/// One admitted read. Opening it spends one of the observation's reads, and
/// only opening it yields the tree, so no read reaches the media unspent.
/// What a caller does before opening it, such as admitting its own path
/// storage, spends nothing.
pub(super) struct ReadAttempt<'tree> {
    tree: ArtifactTreeMedia<'tree>,
    remaining_entries: &'tree mut u64,
}

impl<'tree> ReadAttempt<'tree> {
    pub(super) fn open(self) -> ArtifactTreeMedia<'tree> {
        *self.remaining_entries -= 1;
        self.tree
    }
}

pub(super) fn outcome<D: LimitDimension, F>(
    read: Result<ObservedRecoveryArtifact, ReadStop<D, F>>,
) -> DenialTransitionOutcome<ObservedRecoveryArtifact, ReadRefusal<D>, F> {
    match read {
        Ok(observed) => TransitionOutcome::Success(observed),
        Err(ReadStop::Refused(refusal)) => TransitionOutcome::Denied(refusal),
        Err(ReadStop::Failed(failure)) => TransitionOutcome::Failed(failure),
    }
}

impl<M: DiscoveryMediaBacking> FilesystemObservation<M> {
    /// Reads a whole artifact of at most `ceiling` bytes. The tree is asked
    /// for no more than the ceiling, the grant and this observation's bytes
    /// allow, and reports the artifact's real length when it is longer.
    pub(super) fn read_whole_charged<D: LimitDimension, F: From<ArtifactDamage>>(
        &mut self,
        context: RecoveryDiscoveryArtifact,
        ceiling: u64,
        fixed: bool,
        grant: &ReadGrant<D>,
        read: impl FnOnce(ReadAttempt<'_>, u64) -> Result<Vec<u8>, TreeReadFailure<F>>,
    ) -> Result<ObservedRecoveryArtifact, ReadStop<D, F>> {
        self.admit_read().map_err(ReadStop::stop)?;
        let limit = ceiling
            .min(grant.bytes().unwrap_or(u64::MAX))
            .min(self.remaining_bytes);
        let store = self.parts.store_identity();
        match read(self.attempt(), limit) {
            Ok(bytes) => {
                self.spend_read_bytes(bytes.len() as u64)
                    .map_err(ReadStop::stop)?;
                if fixed {
                    self.counters.fixed_slots_read += 1;
                } else {
                    self.counters.addressed_artifacts_read += 1;
                }
                Ok(ObservedRecoveryArtifact::new(
                    store,
                    context,
                    0,
                    Some(bytes),
                ))
            }
            Err(TreeReadFailure::Media(failure))
                if failure.kind() == ArtifactTreeFailureKind::Absent =>
            {
                // A fixed slot is read whether or not it holds an artifact.
                if fixed {
                    self.counters.fixed_slots_read += 1;
                }
                Ok(ObservedRecoveryArtifact::new(store, context, 0, None))
            }
            Err(TreeReadFailure::Media(failure)) => {
                Err(self.whole_refusal(context, failure, ceiling, grant))
            }
            Err(TreeReadFailure::Caller(failure)) => Err(ReadStop::Failed(failure)),
        }
    }

    /// Reads exactly `length` bytes at `offset`. The range is the read's real
    /// length, so every bound is decided before anything is opened; a file
    /// shorter than the range is the tree's damage.
    pub(super) fn read_range_charged<D: LimitDimension, F: From<ArtifactDamage>>(
        &mut self,
        context: RecoveryDiscoveryArtifact,
        offset: u64,
        length: u32,
        grant: &ReadGrant<D>,
        read: impl FnOnce(ReadAttempt<'_>, usize) -> Result<Vec<u8>, TreeReadFailure<F>>,
    ) -> Result<ObservedRecoveryArtifact, ReadStop<D, F>> {
        let bytes = u64::from(length);
        let capacity = usize::try_from(length).ok();
        let (true, Some(_), Some(capacity)) = (length > 0, offset.checked_add(bytes), capacity)
        else {
            return Err(ReadStop::damage(ArtifactDamage::InvalidAddress {
                artifact: context,
            }));
        };
        if let Some(overrun) = grant.overrun(bytes) {
            return Err(ReadStop::Refused(ReadRefusal::PastGrant(overrun)));
        }
        self.spent_with(bytes).map_err(ReadStop::stop)?;
        self.admit_read().map_err(ReadStop::stop)?;
        let store = self.parts.store_identity();
        match read(self.attempt(), capacity) {
            Ok(observed) => {
                self.spend_read_bytes(bytes).map_err(ReadStop::stop)?;
                self.counters.addressed_artifacts_read += 1;
                Ok(ObservedRecoveryArtifact::new(
                    store,
                    context,
                    offset,
                    Some(observed),
                ))
            }
            Err(TreeReadFailure::Media(failure))
                if failure.kind() == ArtifactTreeFailureKind::Absent =>
            {
                Ok(ObservedRecoveryArtifact::new(store, context, offset, None))
            }
            Err(TreeReadFailure::Media(failure)) => Err(ReadStop::damage(ArtifactDamage::Media {
                artifact: context,
                failure,
            })),
            Err(TreeReadFailure::Caller(failure)) => Err(ReadStop::Failed(failure)),
        }
    }

    fn attempt(&mut self) -> ReadAttempt<'_> {
        ReadAttempt {
            tree: self.parts.artifact_tree(),
            remaining_entries: &mut self.remaining_entries,
        }
    }

    /// A whole read the tree refused at the artifact's real `length`: past
    /// the ceiling, then past the grant, then past this observation's bytes.
    /// A refusal that names no length, or a length every bound admits, is
    /// the tree's own damage.
    fn whole_refusal<D: LimitDimension, F: From<ArtifactDamage>>(
        &self,
        context: RecoveryDiscoveryArtifact,
        failure: ArtifactTreeFailure,
        ceiling: u64,
        grant: &ReadGrant<D>,
    ) -> ReadStop<D, F> {
        let Some(limit) = failure.access_limit() else {
            return ReadStop::damage(ArtifactDamage::Media {
                artifact: context,
                failure,
            });
        };
        let length = limit.observed;
        if length > ceiling {
            return ReadStop::damage(ArtifactDamage::PastCeiling {
                artifact: context,
                length,
                ceiling,
            });
        }
        if let Some(overrun) = grant.overrun(length) {
            return ReadStop::Refused(ReadRefusal::PastGrant(overrun));
        }
        match self.spent_with(length) {
            Err(refused) => ReadStop::stop(refused),
            Ok(_) => ReadStop::damage(ArtifactDamage::Media {
                artifact: context,
                failure,
            }),
        }
    }
}
