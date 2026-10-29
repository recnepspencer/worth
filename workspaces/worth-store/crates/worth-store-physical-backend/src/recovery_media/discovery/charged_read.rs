//! The one engine for reads of whole artifacts, streams and exact ranges.
//! Each refusal is decided here, with the read's real length: from the
//! artifact's metadata for a whole read, from the range itself for a range.
//! Past a declared ceiling is damage; past the caller's grant is that
//! budget's; past the observation's own reads or bytes is the observation's.

use worth_foundational::LimitDimension;
use worth_proof::{DenialTransitionOutcome, TransitionOutcome};

use crate::filesystem_media::{ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeMedia};

use super::super::ceiling::CeilingBytes;
use super::super::grant::{GrantShare, ReadGrant};
use super::super::refusal::{AllocatedReadFailure, ArtifactDamage, ReadRefusal};
use super::{
    commit_read_bytes, DiscoveryMediaBacking, FilesystemObservation, ObservedRecoveryArtifact,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact, RecoveryDiscoveryCount,
    RecoveryDiscoveryCounters, RecoveryDiscoveryFailure,
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

impl<D: LimitDimension, E> From<RecoveryDiscoveryAllocationFailure<E>>
    for ReadStop<D, AllocatedReadFailure<E>>
{
    /// What a read into a caller's buffer met before any grant was asked.
    fn from(failure: RecoveryDiscoveryAllocationFailure<E>) -> Self {
        match failure {
            RecoveryDiscoveryAllocationFailure::Discovery(failure) => Self::stop(failure),
            RecoveryDiscoveryAllocationFailure::Allocation {
                artifact,
                offset,
                requested,
                cause,
            } => Self::Failed(AllocatedReadFailure::Allocation {
                artifact,
                offset,
                requested,
                cause,
            }),
            RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            } => Self::Failed(AllocatedReadFailure::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            }),
        }
    }
}

impl<D: LimitDimension, E> From<RecoveryDiscoveryFailure> for ReadStop<D, AllocatedReadFailure<E>> {
    fn from(failure: RecoveryDiscoveryFailure) -> Self {
        Self::stop(failure)
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

/// The count a read's grant spends, named by the stream being read: one
/// inventory's WAL bytes for a WAL file, the bytes read for every other.
pub(super) const fn granted_count(context: &RecoveryDiscoveryArtifact) -> RecoveryDiscoveryCount {
    match context {
        RecoveryDiscoveryArtifact::WalArtifact(_) => RecoveryDiscoveryCount::WalBytesRead,
        RecoveryDiscoveryArtifact::Record(_)
        | RecoveryDiscoveryArtifact::CurrentCheckpoint
        | RecoveryDiscoveryArtifact::WalDirectory => RecoveryDiscoveryCount::BytesRead,
    }
}

pub(super) fn outcome<T, D: LimitDimension, F>(
    read: Result<T, ReadStop<D, F>>,
) -> DenialTransitionOutcome<T, ReadRefusal<D>, F> {
    match read {
        Ok(observed) => TransitionOutcome::Success(observed),
        Err(ReadStop::Refused(refusal)) => TransitionOutcome::Denied(refusal),
        Err(ReadStop::Failed(failure)) => TransitionOutcome::Failed(failure),
    }
}

impl<M: DiscoveryMediaBacking> FilesystemObservation<M> {
    /// Reads a whole artifact within `ceiling`. The tree is asked for no more
    /// than the ceiling, what is left of the grant and this observation's
    /// bytes allow, and reports the artifact's real length when it is longer.
    pub(super) fn read_whole_charged<D: LimitDimension, F: From<ArtifactDamage>>(
        &mut self,
        context: RecoveryDiscoveryArtifact,
        ceiling: CeilingBytes,
        fixed: bool,
        grant: &GrantShare<'_, D>,
        read: impl FnOnce(ReadAttempt<'_>, u64) -> Result<Vec<u8>, TreeReadFailure<F>>,
    ) -> Result<ObservedRecoveryArtifact, ReadStop<D, F>> {
        self.admit_read().map_err(ReadStop::stop)?;
        let limit = ceiling
            .admitted()
            .min(grant.left().unwrap_or(u64::MAX))
            .min(self.remaining_bytes);
        let mut caller_failure = None;
        let mut read = Some(read);
        match drive_whole_read(
            &mut self.parts,
            &mut self.remaining_entries,
            &mut self.remaining_bytes,
            self.maximum_bytes,
            &mut self.counters,
            context,
            limit,
            fixed,
            &mut |attempt, admitted| match read.take().expect("charged whole read runs once")(
                attempt, admitted,
            ) {
                Ok(bytes) => Ok(bytes),
                Err(TreeReadFailure::Media(failure)) => Err(DriveReadFailure::Media(failure)),
                Err(TreeReadFailure::Caller(failure)) => {
                    caller_failure = Some(failure);
                    Err(DriveReadFailure::Caller)
                }
            },
        ) {
            Ok(artifact) => Ok(artifact),
            Err(DriveWholeStop::Caller) => Err(ReadStop::Failed(
                caller_failure.expect("charged whole read stores its caller failure"),
            )),
            Err(DriveWholeStop::Observation(failure)) => Err(ReadStop::stop(failure)),
            Err(DriveWholeStop::Media { context, failure }) => {
                Err(self.whole_refusal(context, failure, ceiling, grant))
            }
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
        let mut caller_failure = None;
        let mut read = Some(read);
        match drive_range_read(
            &mut self.parts,
            &mut self.remaining_entries,
            &mut self.remaining_bytes,
            self.maximum_bytes,
            &mut self.counters,
            context,
            offset,
            bytes,
            capacity,
            &mut |attempt, capacity| match read.take().expect("charged range read runs once")(
                attempt, capacity,
            ) {
                Ok(observed) => Ok(observed),
                Err(TreeReadFailure::Media(failure)) => Err(DriveReadFailure::Media(failure)),
                Err(TreeReadFailure::Caller(failure)) => {
                    caller_failure = Some(failure);
                    Err(DriveReadFailure::Caller)
                }
            },
        ) {
            Ok(artifact) => Ok(artifact),
            Err(DriveRangeStop::Caller) => Err(ReadStop::Failed(
                caller_failure.expect("charged range read stores its caller failure"),
            )),
            Err(DriveRangeStop::Observation(failure)) => Err(ReadStop::stop(failure)),
            Err(DriveRangeStop::Media { context, failure }) => {
                Err(ReadStop::damage(ArtifactDamage::Media {
                    artifact: context,
                    failure,
                }))
            }
        }
    }

    /// A whole read the tree refused at the artifact's real `length`: past
    /// a declared ceiling, then past what is left of the grant, then past
    /// this observation's bytes. A refusal that names no length, or a length
    /// every bound admits, is the tree's own damage.
    fn whole_refusal<D: LimitDimension, F: From<ArtifactDamage>>(
        &self,
        context: RecoveryDiscoveryArtifact,
        failure: ArtifactTreeFailure,
        ceiling: CeilingBytes,
        grant: &GrantShare<'_, D>,
    ) -> ReadStop<D, F> {
        let Some(limit) = failure.access_limit() else {
            return ReadStop::damage(ArtifactDamage::Media {
                artifact: context,
                failure,
            });
        };
        let length = limit.observed;
        if let Some(ceiling) = ceiling.passed_by(length) {
            return ReadStop::damage(ArtifactDamage::PastCeiling {
                artifact: context,
                length,
                ceiling,
            });
        }
        match grant.overrun(length) {
            Ok(Some(overrun)) => return ReadStop::Refused(ReadRefusal::PastGrant(overrun)),
            Ok(None) => {}
            Err(_) => {
                return ReadStop::damage(ArtifactDamage::CountOverflow(granted_count(&context)))
            }
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

enum DriveReadFailure {
    Media(ArtifactTreeFailure),
    Caller,
}

enum DriveWholeStop {
    Observation(RecoveryDiscoveryFailure),
    Media {
        context: RecoveryDiscoveryArtifact,
        failure: ArtifactTreeFailure,
    },
    Caller,
}

enum DriveRangeStop {
    Observation(RecoveryDiscoveryFailure),
    Media {
        context: RecoveryDiscoveryArtifact,
        failure: ArtifactTreeFailure,
    },
    Caller,
}

/// One compiled whole-artifact read for every backing, limit, and caller closure.
#[inline(never)]
fn drive_whole_read(
    parts: &mut dyn DiscoveryMediaBacking,
    remaining_entries: &mut u64,
    remaining_bytes: &mut u64,
    maximum_bytes: u64,
    counters: &mut RecoveryDiscoveryCounters,
    context: RecoveryDiscoveryArtifact,
    limit: u64,
    fixed: bool,
    read: &mut dyn FnMut(ReadAttempt<'_>, u64) -> Result<Vec<u8>, DriveReadFailure>,
) -> Result<ObservedRecoveryArtifact, DriveWholeStop> {
    let store = parts.store_identity();
    match read(
        ReadAttempt {
            tree: parts.artifact_tree(),
            remaining_entries,
        },
        limit,
    ) {
        Ok(bytes) => {
            commit_read_bytes(counters, remaining_bytes, maximum_bytes, bytes.len() as u64)
                .map_err(DriveWholeStop::Observation)?;
            if fixed {
                counters.fixed_slots_read += 1;
            } else {
                counters.addressed_artifacts_read += 1;
            }
            Ok(ObservedRecoveryArtifact::new(
                store,
                context,
                0,
                Some(bytes),
            ))
        }
        Err(DriveReadFailure::Media(failure))
            if failure.kind() == ArtifactTreeFailureKind::Absent =>
        {
            // A fixed slot is read whether or not it holds an artifact.
            if fixed {
                counters.fixed_slots_read += 1;
            }
            Ok(ObservedRecoveryArtifact::new(store, context, 0, None))
        }
        Err(DriveReadFailure::Media(failure)) => Err(DriveWholeStop::Media { context, failure }),
        Err(DriveReadFailure::Caller) => Err(DriveWholeStop::Caller),
    }
}

/// One compiled exact-range read for every backing, limit, and caller closure.
#[inline(never)]
fn drive_range_read(
    parts: &mut dyn DiscoveryMediaBacking,
    remaining_entries: &mut u64,
    remaining_bytes: &mut u64,
    maximum_bytes: u64,
    counters: &mut RecoveryDiscoveryCounters,
    context: RecoveryDiscoveryArtifact,
    offset: u64,
    bytes: u64,
    capacity: usize,
    read: &mut dyn FnMut(ReadAttempt<'_>, usize) -> Result<Vec<u8>, DriveReadFailure>,
) -> Result<ObservedRecoveryArtifact, DriveRangeStop> {
    let store = parts.store_identity();
    match read(
        ReadAttempt {
            tree: parts.artifact_tree(),
            remaining_entries,
        },
        capacity,
    ) {
        Ok(observed) => {
            commit_read_bytes(counters, remaining_bytes, maximum_bytes, bytes)
                .map_err(DriveRangeStop::Observation)?;
            counters.addressed_artifacts_read += 1;
            Ok(ObservedRecoveryArtifact::new(
                store,
                context,
                offset,
                Some(observed),
            ))
        }
        Err(DriveReadFailure::Media(failure))
            if failure.kind() == ArtifactTreeFailureKind::Absent =>
        {
            Ok(ObservedRecoveryArtifact::new(store, context, offset, None))
        }
        Err(DriveReadFailure::Media(failure)) => Err(DriveRangeStop::Media { context, failure }),
        Err(DriveReadFailure::Caller) => Err(DriveRangeStop::Caller),
    }
}
