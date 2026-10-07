//! Why a record has no reusable computation, carried from the dropping site.

use super::{SealedComputationRun, WorthQueryPartitionedComputationFullCause as Cause};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum PriorAbsence {
    FirstRun,
    Restored,
    Republished,
    NotProduced,
    Unmeasured,
    Moved,
    Evicted,
    Suppressed(Suppression),
    Stopped,
}

impl PriorAbsence {
    pub(in crate::domain_computation) const fn full_cause(&self) -> Cause {
        match self {
            Self::FirstRun => Cause::FirstRun,
            Self::Restored => Cause::Restored,
            Self::Republished => Cause::Republished,
            Self::NotProduced => Cause::NotProduced,
            Self::Unmeasured => Cause::Unmeasured,
            Self::Moved => Cause::Moved,
            Self::Evicted => Cause::Evicted,
            Self::Suppressed(Suppression::Policy) => Cause::RetentionPolicy,
            Self::Suppressed(Suppression::Several) => Cause::SeveralComputations,
            Self::Suppressed(Suppression::Collision) => Cause::CollisionSuppressed,
            Self::Stopped => Cause::Stopped,
        }
    }
}

/// Completion must state an absence explicitly; publication cannot infer it.
pub(in crate::domain_computation) enum SealedComputationRetention {
    Produced(SealedComputationRun),
    Absent(PriorAbsence),
}

/// Why a completed computation could not leave one reusable state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum Suppression {
    Policy,
    Several,
    Collision,
}

/// A run writes its result where it completes or drops its state.
pub(in crate::domain_computation::primary_graph) enum CompletedComputationRetention {
    Produced(super::retained::CompletedComputationRun),
    Absent(PriorAbsence),
}
