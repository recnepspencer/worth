//! A computational failure is transported only from the call that owns its cause.
use worth_execution::NestedComputationStop;

pub(in crate::domain_computation::primary_graph) struct NestedFailureTransport<'context> {
    take: &'context mut dyn FnMut() -> Option<NestedComputationStop>,
}

pub(in crate::domain_computation::primary_graph) enum MemberComputeFailure<E> {
    Own(E),
    Nested(TakenNestedFailure<E>),
}

/// Construction requires the value removed from the execution owner's latch.
pub(in crate::domain_computation::primary_graph) struct TakenNestedFailure<E> {
    cause: NestedComputationStop,
    failure: E,
}

impl<'context> NestedFailureTransport<'context> {
    pub(in crate::domain_computation::primary_graph) fn new(
        take: &'context mut dyn FnMut() -> Option<NestedComputationStop>,
    ) -> Self {
        Self { take }
    }

    /// Run and classify one nested call before any subsequent call can start.
    /// A request-owned cause yields no value and remains latched.
    pub(in crate::domain_computation::primary_graph) fn call<T, E>(
        &mut self,
        run: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, MemberComputeFailure<E>> {
        run().map_err(|failure| match (self.take)() {
            Some(cause) => MemberComputeFailure::Nested(TakenNestedFailure { cause, failure }),
            None => MemberComputeFailure::Own(failure),
        })
    }
}

impl<E> From<E> for MemberComputeFailure<E> {
    fn from(failure: E) -> Self {
        Self::Own(failure)
    }
}

impl<E> MemberComputeFailure<E> {
    /// The owner's public denial already carries the specific value and identity.
    /// Consume its transport proof only as the completed request becomes data.
    pub(in crate::domain_computation::primary_graph) fn into_failure(self) -> E {
        match self {
            Self::Own(failure) => failure,
            Self::Nested(TakenNestedFailure { cause, failure }) => match cause {
                NestedComputationStop::DomainFailure
                | NestedComputationStop::Allocation(_)
                | NestedComputationStop::Panic
                | NestedComputationStop::ResultCapacityExceeded
                | NestedComputationStop::WorkCeiling => failure,
            },
        }
    }
}
