use super::super::{CausalLinkKind, MeaningfulChangeReason, UpstreamCause};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for MeaningfulChangeReason {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::CustomComparator { key } => key.retained_heap_charge(work),
            Self::ExactDifference
            | Self::Tolerance { epsilon: _ }
            | Self::OutputIdentity
            | Self::InstalledComparator
            | Self::InheritedComparator => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for CausalLinkKind {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::ConditionDeferred {
                condition,
                decision: _,
            } => condition.retained_heap_charge(work),
            Self::Changed
            | Self::SkippedByComparator
            | Self::ScopeUntouched
            | Self::Clean
            | Self::MissingSnapshot
            | Self::DependencyAdded
            | Self::DependencyRemoved => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for UpstreamCause {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Changed {
                source: _,
                aspect: _,
                subscription,
                cached_version: _,
                current_version: _,
                comparator,
                reason,
            }
            | Self::SkippedByComparator {
                source: _,
                aspect: _,
                subscription,
                cached_version: _,
                current_version: _,
                comparator,
                reason,
            } => subscription
                .retained_heap_charge(work)?
                .checked_add(comparator.retained_heap_charge(work)?)?
                .checked_add(reason.retained_heap_charge(work)?),
            Self::ConditionDeferred {
                source: _,
                aspect: _,
                subscription,
                cached_version: _,
                current_version: _,
                condition,
                decision: _,
            } => subscription
                .retained_heap_charge(work)?
                .checked_add(condition.retained_heap_charge(work)?),
            Self::Clean {
                source: _,
                aspect: _,
                subscription,
                cached_version: _,
                current_version: _,
            }
            | Self::MissingSnapshot {
                source: _,
                aspect: _,
                subscription,
                current_version: _,
            }
            | Self::DependencyRemoved {
                source: _,
                aspect: _,
                subscription,
                cached_version: _,
            } => subscription.retained_heap_charge(work),
        }
    }
}
