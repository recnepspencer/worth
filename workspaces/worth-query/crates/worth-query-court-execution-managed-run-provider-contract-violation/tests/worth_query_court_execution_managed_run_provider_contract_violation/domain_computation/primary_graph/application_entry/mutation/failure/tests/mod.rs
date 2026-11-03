use worth_query_execution::facade::domain_computation::HandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::HandlerFailure;
use worth_query_execution::facade::domain_computation::HandlerInterruption;
use worth_query_execution::facade::domain_computation::HandlerResult;

#[derive(Debug, PartialEq)]
pub struct OriginalSource(u64);
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

    let source =
        HandlerResult::<(), u32>::ExecutionDenied(HandlerExecutionDenial::new(OriginalSource(91)))
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
