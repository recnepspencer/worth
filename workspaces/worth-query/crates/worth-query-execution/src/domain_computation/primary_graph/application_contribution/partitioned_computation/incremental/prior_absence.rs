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
            Self::Suppressed(Suppression::NoProducerPrior) => Cause::NoProducerPrior,
            Self::Suppressed(Suppression::PriorAlreadyTaken) => Cause::NoPriorHanded,
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
    NoProducerPrior,
    PriorAlreadyTaken,
}

/// A run writes its result where it completes or drops its state.
pub(in crate::domain_computation::primary_graph) enum CompletedComputationRetention {
    Produced(super::retained::CompletedComputationRun),
    Absent(PriorAbsence),
}

impl CompletedComputationRetention {
    pub(in super::super) fn transport_bytes(&self, tree_inline: u64) -> u64 {
        match self {
            Self::Absent(_) => 0,
            Self::Produced(run) => run
                .typed_bytes
                .unwrap_or(u64::MAX)
                .saturating_sub(run.tree_memory.bytes().saturating_sub(tree_inline))
                // The sequence carries its own grants through result handoff;
                // counting it here again would reject lawful small-partition
                // results. Its full resident charge remains in typed_bytes.
                .saturating_sub(run.call_sequence.funded_bytes())
                .saturating_sub(run.routing_memory.as_ref().map_or(0, |held| held.bytes())),
        }
    }
}
