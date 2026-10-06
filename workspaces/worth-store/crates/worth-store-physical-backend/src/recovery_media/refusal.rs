//! What a read of a whole artifact or an exact range met, classified where it
//! was refused and with the read's real length. A budget the caller set, or the
//! observation's own, refuses (`Denied`); what no budget can fix is damage
//! (`Failed`).

use worth_foundational::LimitDimension;
use worth_proof::{DenialTransitionOutcome, TransitionOutcome};

use super::discovery::{
    ExceededFilesystemObservationBound, ObservedRecoveryArtifact,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact, RecoveryDiscoveryCount,
    RecoveryDiscoveryFailure,
};
use super::grant::{GrantOverrun, Uncharged};
use crate::filesystem_media::ArtifactTreeFailure;

/// A budget refused the read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadRefusal<D: LimitDimension> {
    /// The read's real length passed the caller's grant: the grant's owner
    /// states the limit.
    PastGrant(GrantOverrun<D>),
    /// The observation's own reads or bytes ran out.
    Observation(ExceededFilesystemObservationBound),
}

/// What more budget cannot fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactDamage {
    /// The artifact is `length` bytes, longer than the `ceiling` its format
    /// declares.
    PastCeiling {
        artifact: RecoveryDiscoveryArtifact,
        length: u64,
        ceiling: u64,
    },
    Media {
        artifact: RecoveryDiscoveryArtifact,
        failure: ArtifactTreeFailure,
    },
    InvalidAddress {
        artifact: RecoveryDiscoveryArtifact,
    },
    /// A count this observation keeps went past every count. No budget admits
    /// it, so it is no limit.
    CountOverflow(RecoveryDiscoveryCount),
}

/// What a read into a caller's buffer met that is no budget's refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocatedReadFailure<E> {
    Damage(ArtifactDamage),
    Allocation {
        artifact: RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: E,
    },
    BufferLengthMismatch {
        artifact: RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        observed: usize,
    },
}

impl<E> From<ArtifactDamage> for AllocatedReadFailure<E> {
    fn from(damage: ArtifactDamage) -> Self {
        Self::Damage(damage)
    }
}

pub type ArtifactReadOutcome<D> =
    DenialTransitionOutcome<ObservedRecoveryArtifact, ReadRefusal<D>, ArtifactDamage>;

pub type AllocatedReadOutcome<D, E> =
    DenialTransitionOutcome<ObservedRecoveryArtifact, ReadRefusal<D>, AllocatedReadFailure<E>>;

/// A read no caller budgets, stated as the observation's result: it can only
/// stop at the observation's own bound or at damage. `Output` is what the
/// read produced: the observed artifact, or whatever a caller wrapped it in.
pub trait UnchargedRead {
    type Output;
    type Failure;

    fn observed(self) -> Result<Self::Output, Self::Failure>;
}

impl<S> UnchargedRead for DenialTransitionOutcome<S, ReadRefusal<Uncharged>, ArtifactDamage> {
    type Output = S;
    type Failure = RecoveryDiscoveryFailure;

    fn observed(self) -> Result<S, RecoveryDiscoveryFailure> {
        match self {
            TransitionOutcome::Success(observed) => Ok(observed),
            TransitionOutcome::Denied(refusal) => Err(uncharged_limit(refusal)),
            TransitionOutcome::Failed(damage) => Err(RecoveryDiscoveryFailure::Damage(damage)),
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never) => match never {},
        }
    }
}

impl<S, E> UnchargedRead
    for DenialTransitionOutcome<S, ReadRefusal<Uncharged>, AllocatedReadFailure<E>>
{
    type Output = S;
    type Failure = RecoveryDiscoveryAllocationFailure<E>;

    fn observed(self) -> Result<S, RecoveryDiscoveryAllocationFailure<E>> {
        match self {
            TransitionOutcome::Success(observed) => Ok(observed),
            TransitionOutcome::Denied(refusal) => Err(
                RecoveryDiscoveryAllocationFailure::Discovery(uncharged_limit(refusal)),
            ),
            TransitionOutcome::Failed(failure) => Err(failure.into()),
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never) => match never {},
        }
    }
}

/// What stopped a granted read: its grant, which the grant's owner states as
/// its own limit, or what an uncharged read would have met.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantedReadStop<D: LimitDimension, F> {
    PastGrant(GrantOverrun<D>),
    Unread(F),
}

/// A read a caller budgets, stated as the observation's result with the
/// grant's overrun set apart for the grant's owner.
pub trait GrantedRead {
    type Output;
    type Dimension: LimitDimension;
    type Failure;

    fn granted(self) -> Result<Self::Output, GrantedReadStop<Self::Dimension, Self::Failure>>;
}

impl<S, D: LimitDimension> GrantedRead
    for DenialTransitionOutcome<S, ReadRefusal<D>, ArtifactDamage>
{
    type Output = S;
    type Dimension = D;
    type Failure = RecoveryDiscoveryFailure;

    fn granted(self) -> Result<S, GrantedReadStop<D, RecoveryDiscoveryFailure>> {
        match self {
            TransitionOutcome::Success(observed) => Ok(observed),
            TransitionOutcome::Denied(ReadRefusal::PastGrant(overrun)) => {
                Err(GrantedReadStop::PastGrant(overrun))
            }
            TransitionOutcome::Denied(ReadRefusal::Observation(limit)) => Err(
                GrantedReadStop::Unread(RecoveryDiscoveryFailure::Limit(limit)),
            ),
            TransitionOutcome::Failed(damage) => Err(GrantedReadStop::Unread(
                RecoveryDiscoveryFailure::Damage(damage),
            )),
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never) => match never {},
        }
    }
}

impl<S, D: LimitDimension, E> GrantedRead
    for DenialTransitionOutcome<S, ReadRefusal<D>, AllocatedReadFailure<E>>
{
    type Output = S;
    type Dimension = D;
    type Failure = RecoveryDiscoveryAllocationFailure<E>;

    fn granted(self) -> Result<S, GrantedReadStop<D, RecoveryDiscoveryAllocationFailure<E>>> {
        match self {
            TransitionOutcome::Success(observed) => Ok(observed),
            TransitionOutcome::Denied(ReadRefusal::PastGrant(overrun)) => {
                Err(GrantedReadStop::PastGrant(overrun))
            }
            TransitionOutcome::Denied(ReadRefusal::Observation(limit)) => Err(
                GrantedReadStop::Unread(RecoveryDiscoveryAllocationFailure::Discovery(
                    RecoveryDiscoveryFailure::Limit(limit),
                )),
            ),
            TransitionOutcome::Failed(failure) => Err(GrantedReadStop::Unread(failure.into())),
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never) => match never {},
        }
    }
}

fn uncharged_limit(refusal: ReadRefusal<Uncharged>) -> RecoveryDiscoveryFailure {
    match refusal {
        ReadRefusal::PastGrant(overrun) => overrun.impossible(),
        ReadRefusal::Observation(limit) => RecoveryDiscoveryFailure::Limit(limit),
    }
}

impl<E> From<AllocatedReadFailure<E>> for RecoveryDiscoveryAllocationFailure<E> {
    fn from(failure: AllocatedReadFailure<E>) -> Self {
        match failure {
            AllocatedReadFailure::Damage(damage) => {
                Self::Discovery(RecoveryDiscoveryFailure::Damage(damage))
            }
            AllocatedReadFailure::Allocation {
                artifact,
                offset,
                requested,
                cause,
            } => Self::Allocation {
                artifact,
                offset,
                requested,
                cause,
            },
            AllocatedReadFailure::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            } => Self::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            },
        }
    }
}
