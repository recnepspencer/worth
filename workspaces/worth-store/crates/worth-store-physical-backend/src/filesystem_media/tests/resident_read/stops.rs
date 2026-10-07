//! Where a read stopped, named the same way for every reader.

use super::*;

/// Where a read stopped, as a test names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Stop {
    Observation(FilesystemObservationBound, u64, u64),
    PastGrant { granted: u64, length: u64 },
    PastCeiling { length: u64, ceiling: u64 },
    Media(ArtifactTreeFailureKind),
    InvalidAddress,
    CountOverflow,
    Allocation,
    BufferLengthMismatch,
}

pub(super) trait Stopped {
    fn stop(&self) -> Stop;
}

impl Stopped for ArtifactDamage {
    fn stop(&self) -> Stop {
        match self {
            Self::PastCeiling {
                length, ceiling, ..
            } => Stop::PastCeiling {
                length: *length,
                ceiling: *ceiling,
            },
            Self::Media { failure, .. } => Stop::Media(failure.kind()),
            Self::InvalidAddress { .. } => Stop::InvalidAddress,
            Self::CountOverflow(_) => Stop::CountOverflow,
        }
    }
}

impl<E> Stopped for AllocatedReadFailure<E> {
    fn stop(&self) -> Stop {
        match self {
            Self::Damage(damage) => damage.stop(),
            Self::Allocation { .. } => Stop::Allocation,
            Self::BufferLengthMismatch { .. } => Stop::BufferLengthMismatch,
        }
    }
}

/// Where a charged read stopped; `None` for a read that returned.
pub(super) fn stopped<S, D: worth_foundational::LimitDimension, F: Stopped>(
    outcome: &worth_proof::DenialTransitionOutcome<S, ReadRefusal<D>, F>,
) -> Option<Stop> {
    match outcome {
        TransitionOutcome::Success(_) => None,
        TransitionOutcome::Denied(ReadRefusal::PastGrant(overrun)) => Some(Stop::PastGrant {
            granted: overrun.granted(),
            length: overrun.length(),
        }),
        TransitionOutcome::Denied(ReadRefusal::Observation(limit)) => Some(Stop::Observation(
            limit.dimension(),
            limit.observed(),
            limit.admitted(),
        )),
        TransitionOutcome::Failed(failure) => Some(failure.stop()),
        TransitionOutcome::Deferred(never)
        | TransitionOutcome::Stale(never)
        | TransitionOutcome::RebindRequired(never) => match *never {},
    }
}

pub(super) fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

/// One page of the test format.
pub(super) fn page() -> u64 {
    u64::from(format().page_size().bytes())
}

pub(super) fn root_ceiling(generation: u64) -> ArtifactCeiling {
    ArtifactCeiling::page(format(), PageAddress::RootManifest { generation })
}

/// The checkpoint stream, whose length no fact declares.
pub(super) fn checkpoint() -> ArtifactCeiling {
    ArtifactCeiling::undeclared(crate::recovery_media::StreamArtifact::CurrentCheckpoint)
}

pub(super) fn uncharged() -> ReadGrant<crate::recovery_media::Uncharged> {
    ReadGrant::ceiling_only()
}
