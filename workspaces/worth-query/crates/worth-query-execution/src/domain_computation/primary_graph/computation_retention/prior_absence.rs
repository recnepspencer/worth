//! Why a record has no reusable computation, carried from the dropping site.

use super::full_cause::WorthQueryPartitionedComputationFullCause as Cause;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PriorAbsence {
    FirstRun,
    Restored,
    Republished,
    NotProduced,
    Unmeasured,

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
            Self::Suppressed(Suppression::Policy) => Cause::RetentionPolicy,
            Self::Suppressed(Suppression::Several) => Cause::SeveralComputations,
            Self::Suppressed(Suppression::Collision) => Cause::CollisionSuppressed,
            Self::Suppressed(Suppression::NoProducerPrior) => Cause::NoProducerPrior,
            Self::Suppressed(Suppression::PriorAlreadyTaken) => Cause::NoPriorHanded,
            Self::Stopped => Cause::Stopped,
        }
    }
}

/// Why a completed computation could not leave one reusable state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suppression {
    Policy,
    Several,
    Collision,
    NoProducerPrior,
    PriorAlreadyTaken,
}

mod court_doors_for_test;
