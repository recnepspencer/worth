//! The exact candidate registry is a nonwaiting reversible close preflight.
use super::*;
use crate::change_source::{RelationalMemoryCloseObservation as O, RelationalRuntimeHandle};
use crate::tests::support::runtime_with_test_schema;

#[test]
fn memory_terminal_close_publication_registry_contention_preserves_admission() {
    let host = crate::test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let publication = runtime.publication_binding();
    let binding = runtime.owner_binding();
    let mut close = RelationalRuntimeHandle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    let locked = publication.lifecycle.candidates.lock().unwrap();
    assert!(matches!(close.advance(request), O::Busy));
    drop(
        binding
            .admit()
            .expect("no irreversible cleanup on busy preflight"),
    );
    assert!(locked.is_empty());
    drop(locked);
    for _ in 0..20 {
        match close.advance(request) {
            O::Advanced => {}
            O::Complete => break,
            other => panic!("unexpected real owner progress: {other:?}"),
        }
    }
    assert!(matches!(close.advance(request), O::Complete));
    assert!(binding.admit().is_none());
    close.into_retired_memory().unwrap().reclaim();
}
