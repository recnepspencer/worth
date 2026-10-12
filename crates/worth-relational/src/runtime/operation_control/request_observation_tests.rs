//! Borrowed request control is observed without creating request authority.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn request_observation_retains_both_control_stops_at_its_boundary() {
    let source = CancellationSource::new();
    let serial = SerialRequest::from_memory(SerialMemoryBudget::new(1024), source.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let boundary = RelationalInterruptionBoundary::ObservationAdmission;
    assert_eq!(
        RelationalInterruptionEvent::observe_request(&request, boundary),
        None
    );
    source.cancel();
    let cancelled = RelationalInterruptionEvent::observe_request(&request, boundary).unwrap();
    assert_eq!(
        cancelled.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(cancelled.boundary(), boundary);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(1024),
        worth_execution::CancellationToken::new(),
        Some(std::time::Instant::now()),
    );
    let expired =
        RelationalInterruptionEvent::observe_request(&ExecutionRequest::serial(&serial), boundary)
            .unwrap();
    assert_eq!(
        expired.interruption(),
        RelationalOperationInterruption::TimedOut
    );
    assert_eq!(expired.boundary(), boundary);
}
