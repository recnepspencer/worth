//! Authentic tenure and the real admission handshake, not Arc-count authority.
use super::*;
use crate::tests::support::runtime_with_test_schema;

#[test]
fn memory_terminal_close_refuses_admitted_tenure_and_retains_actual_inflight_operation() {
    let host = crate::test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let binding = runtime.owner_binding();
    let operation = binding.admit().unwrap();
    let admitted =
        RelationalRuntime::admitted(runtime.state_binding().upgrade().unwrap(), operation);
    let refused = RelationalRuntimeHandle::immutable(Arc::new(admitted))
        .into_memory_close()
        .unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalMemoryCloseDenial::Native(RelationalRuntimeCloseDenial::NotOwner)
    );
    let mut close = RelationalRuntimeHandle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    assert!(matches!(
        close.advance(request),
        RelationalMemoryCloseObservation::Refused(RelationalRuntimeCloseDenial::OperationsInFlight)
    ));
    assert!(!close.sealed);
    drop(
        binding
            .admit()
            .expect("reversible preflight restored actual admission"),
    );
    drop(refused); // Releases the SAME admitted tenure, not a fabricated count.
    for _ in 0..20 {
        match close.advance(request) {
            RelationalMemoryCloseObservation::Advanced => {}
            RelationalMemoryCloseObservation::Complete => break,
            other => panic!("unexpected close result: {other:?}"),
        }
    }
    assert!(close.complete);
    assert!(binding.admit().is_none());
    close.into_retired_memory().unwrap().reclaim();
}
