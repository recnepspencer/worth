//! Computation failures enter the handler topology without rendering their source.

use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerExecutionDenial, HandlerFailure, HandlerInterruption,
};

impl<Domain> From<WorthQueryManagedComputationInterruption> for HandlerFailure<Domain> {
    fn from(cause: WorthQueryManagedComputationInterruption) -> Self {
        Self::Interrupted(match cause {
            WorthQueryManagedComputationInterruption::Cancelled => HandlerInterruption::Cancelled,
            WorthQueryManagedComputationInterruption::DeadlineExceeded => {
                HandlerInterruption::DeadlineExceeded
            }
        })
    }
}

impl<Domain, Stopped> From<WorthQueryManagedComputationDenial<Stopped>> for HandlerFailure<Domain>
where
    Stopped: Into<HandlerFailure<Domain>>,
{
    fn from(cause: WorthQueryManagedComputationDenial<Stopped>) -> Self {
        match cause.into_owner_result() {
            Ok(cause) => cause.into(),
            Err(native) => native.map_domain(|never| match never {}),
        }
    }
}

impl<Stopped> WorthQueryManagedComputationDenial<Stopped> {
    /// Separates a deliberate owner outcome from native request failure when
    /// an adapter gives that owner outcome a domain-specific success meaning.
    pub fn into_owner_result(self) -> Result<Stopped, HandlerFailure<std::convert::Infallible>> {
        match self {
            Self::Owner(cause) => Ok(cause),
            Self::Interrupted(cause) => Err(cause.into()),
            Self::Panicked => Err(HandlerFailure::Execution(HandlerExecutionDenial::new(
                WorthQueryManagedComputationDenial::<std::convert::Infallible>::Panicked,
            ))),
            Self::Resource(cause) => {
                Err(HandlerFailure::Execution(HandlerExecutionDenial::new(
                    WorthQueryManagedComputationDenial::<std::convert::Infallible>::Resource(cause),
                )))
            }
            Self::NestedPatternStopped => {
                Err(HandlerFailure::Execution(HandlerExecutionDenial::new(
                    WorthQueryManagedComputationDenial::<std::convert::Infallible>::NestedPatternStopped,
                )))
            }
        }
    }
}

impl<Domain, Stopped> From<WorthQueryPartitionedComputationDenial<Stopped>>
    for HandlerFailure<Domain>
where
    Stopped: Into<HandlerFailure<Domain>> + std::fmt::Debug + Send + Sync + 'static,
{
    fn from(cause: WorthQueryPartitionedComputationDenial<Stopped>) -> Self {
        match cause {
            WorthQueryPartitionedComputationDenial::Owner(cause)
            | WorthQueryPartitionedComputationDenial::Partition {
                cause: WorthQueryComputationPartitionStop::Owner(cause),
                ..
            } => cause.into(),
            WorthQueryPartitionedComputationDenial::Interrupted(cause)
            | WorthQueryPartitionedComputationDenial::Partition {
                cause: WorthQueryComputationPartitionStop::Interrupted(cause),
                ..
            } => cause.into(),
            cause => Self::Execution(HandlerExecutionDenial::new(cause)),
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn managed_advancement_denial<Stopped>(
    cause: super::super::WorthQueryAdvancementDenial,
) -> WorthQueryManagedComputationDenial<Stopped> {
    match cause {
        super::super::WorthQueryAdvancementDenial::Resource(cause) => {
            WorthQueryManagedComputationDenial::Resource(cause)
        }
        super::super::WorthQueryAdvancementDenial::Interrupted(cause) => {
            WorthQueryManagedComputationDenial::Interrupted(cause)
        }
        super::super::WorthQueryAdvancementDenial::NestedStopped => {
            WorthQueryManagedComputationDenial::NestedPatternStopped
        }
        super::super::WorthQueryAdvancementDenial::NestedOpening => {
            WorthQueryManagedComputationDenial::Resource(Resource::NestedAdvancementOpening)
        }
        super::super::WorthQueryAdvancementDenial::Panicked => {
            WorthQueryManagedComputationDenial::Panicked
        }
    }
}
