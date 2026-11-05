//! Typed failure propagation across handler-owned preparation and completion.

use {
    super::HandlerExecutionDenial, super::HandlerResult,
    crate::domain_computation::primary_graph::vocabulary::handler::invariant::HandlerInterruption,
};

/// A domain refusal retains its meaning; execution and request interruption
/// remain distinct through composition of handler work.
#[derive(Debug)]
pub enum HandlerFailure<Domain> {
    Domain(Domain),
    Execution(HandlerExecutionDenial),
    Interrupted(HandlerInterruption),
}

impl<Domain> HandlerFailure<Domain> {
    /// Changes only domain meaning, preserving the original execution source
    /// and the request's interruption.
    pub fn map_domain<Next>(self, map: impl FnOnce(Domain) -> Next) -> HandlerFailure<Next> {
        match self {
            Self::Domain(domain) => HandlerFailure::Domain(map(domain)),
            Self::Execution(cause) => HandlerFailure::Execution(cause),
            Self::Interrupted(cause) => HandlerFailure::Interrupted(cause),
        }
    }
}

impl<Domain> From<HandlerExecutionDenial> for HandlerFailure<Domain> {
    fn from(cause: HandlerExecutionDenial) -> Self {
        Self::Execution(cause)
    }
}

impl<Domain> From<HandlerInterruption> for HandlerFailure<Domain> {
    fn from(cause: HandlerInterruption) -> Self {
        Self::Interrupted(cause)
    }
}

impl<Domain> From<std::convert::Infallible> for HandlerFailure<Domain> {
    fn from(cause: std::convert::Infallible) -> Self {
        match cause {}
    }
}

impl<Value, Domain> HandlerResult<Value, Domain> {
    pub fn from_result(result: Result<Value, HandlerFailure<Domain>>) -> Self {
        match result {
            Ok(value) => Self::Completed(value),
            Err(HandlerFailure::Domain(cause)) => Self::DomainDenied(cause),
            Err(HandlerFailure::Execution(cause)) => Self::ExecutionDenied(cause),
            Err(HandlerFailure::Interrupted(HandlerInterruption::Cancelled)) => Self::Cancelled,
            Err(HandlerFailure::Interrupted(HandlerInterruption::DeadlineExceeded)) => {
                Self::DeadlineExceeded
            }
        }
    }

    pub fn into_result(self) -> Result<Value, HandlerFailure<Domain>> {
        match self {
            Self::Completed(value) => Ok(value),
            Self::DomainDenied(cause) => Err(HandlerFailure::Domain(cause)),
            Self::ExecutionDenied(cause) => Err(HandlerFailure::Execution(cause)),
            Self::Cancelled => Err(HandlerFailure::Interrupted(HandlerInterruption::Cancelled)),
            Self::DeadlineExceeded => Err(HandlerFailure::Interrupted(
                HandlerInterruption::DeadlineExceeded,
            )),
        }
    }
}
