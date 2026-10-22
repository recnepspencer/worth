//! Typed failure propagation across handler-owned preparation and completion.

use super::{HandlerExecutionDenial, HandlerInterruption, HandlerResult};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct OriginalSource(u64);
    impl std::fmt::Display for OriginalSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "source {}", self.0)
        }
    }
    impl std::error::Error for OriginalSource {}

    #[test]
    fn composition_changes_only_domain_meaning_and_keeps_concrete_execution_source() {
        let domain = HandlerResult::<(), _>::DomainDenied(7)
            .into_result()
            .map_err(|failure| failure.map_domain(|value| value.to_string()));
        assert!(
            matches!(HandlerResult::from_result(domain), HandlerResult::DomainDenied(value) if value == "7")
        );

        let source = HandlerResult::<(), u32>::ExecutionDenied(HandlerExecutionDenial::new(
            OriginalSource(91),
        ))
        .into_result()
        .map_err(|failure| failure.map_domain(|value| value.to_string()));
        let HandlerResult::ExecutionDenied(source) = HandlerResult::from_result(source) else {
            panic!("execution cannot become a domain failure");
        };
        assert_eq!(
            source.downcast::<OriginalSource>().unwrap(),
            OriginalSource(91)
        );
        for outcome in [
            HandlerResult::<(), u32>::Cancelled,
            HandlerResult::DeadlineExceeded,
        ] {
            let interrupted = outcome.into_result().unwrap_err();
            let expected = match &interrupted {
                HandlerFailure::Interrupted(cause) => *cause,
                _ => panic!("a request stop keeps its topology"),
            };
            let result = HandlerResult::<(), _>::from_result(Err(
                interrupted.map_domain(|value| value.to_string())
            ));
            assert!(matches!(
                (expected, result),
                (HandlerInterruption::Cancelled, HandlerResult::Cancelled)
                    | (
                        HandlerInterruption::DeadlineExceeded,
                        HandlerResult::DeadlineExceeded
                    )
            ));
        }
    }
}
