//! Authentic tenure and the real admission handshake, not Arc-count authority.
use super::*;
use crate::change_source::memory_close_observation::RelationalMemoryCloseObservation as O;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};

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

// Exact Runtime admission and owner close are needed to observe these private custody transitions.
#[test]
fn memory_terminal_close_publication_registry_contention_preserves_admission() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let publication = runtime.publication_binding();
    let binding = runtime.owner_binding();
    let mut close = RelationalRuntimeHandle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    publication.while_empty_candidate_registry_locked(|| {
        assert!(matches!(close.advance(request), O::Busy));
        drop(
            binding
                .admit()
                .expect("no irreversible cleanup on busy preflight"),
        );
    });
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

#[test]
fn memory_terminal_close_cache_contention_preserves_exact_and_historical_allocations() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let original = create_entity_outcome(request, &runtime, "original-cache-root");
    runtime
        .read_truth()
        .read_snapshot(&original.snapshot, request)
        .unwrap()
        .unwrap();
    let root = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    let weak_root = std::sync::Arc::downgrade(&root);
    drop(root);
    runtime
        .snapshots()
        .release_snapshot(&original.snapshot)
        .unwrap();
    let next = create_entity_outcome(request, &runtime, "next-cache-root");
    runtime
        .read_truth()
        .read_snapshot(&next.snapshot, request)
        .unwrap()
        .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&next.snapshot)
        .unwrap();
    let basis = crate::visibility::snapshot_states::HistoricalVisibilityBasis::resolve(
        &runtime,
        original.commit.version_id,
    )
    .unwrap();
    crate::visibility::cache_state::ensure_historical_state(request, &runtime, basis, true)
        .unwrap();
    drop(original);
    drop(next);
    let cache = runtime.visibility.cache.clone();
    let original_entries = cache.memory_close_entry_count();
    assert!(original_entries >= 2);
    let mut close = RelationalRuntimeHandle::immutable(std::sync::Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    assert!(matches!(close.advance(request), O::Advanced));
    cache.while_live_entries_locked(original_entries, || {
        assert!(matches!(close.advance(request), O::Busy));
    });
    let mut complete = false;
    for _ in 0..100 {
        match close.advance(request) {
            O::Advanced => {}
            O::Complete => {
                complete = true;
                break;
            }
            other => panic!("actual cache custody must retire: {other:?}"),
        }
    }
    assert!(complete);
    let key = cache.assert_entries_retired(original_entries);
    assert!(
        cache.state(&key).is_none(),
        "retired allocations cannot be read as a synthetic basis"
    );
    let retired = close.into_retired_memory().unwrap();
    assert!(
        weak_root.upgrade().is_some(),
        "genuine old root survives business retirement"
    );
    drop(cache);
    retired.reclaim();
    assert!(
        weak_root.upgrade().is_none(),
        "explicit unbudgeted destruction releases heap"
    );
}

#[test]
fn memory_terminal_close_head_contention_retires_actual_token_without_dropping_root() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let cell = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap();
    let head = cell.head_retention().clone();
    let root = cell.root().unwrap();
    let weak_root = Arc::downgrade(&root);
    drop(root);
    drop(cell);
    let mut close = RelationalRuntimeHandle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    assert!(matches!(close.advance(request), O::Advanced));
    assert!(matches!(close.advance(request), O::Advanced)); // empty cache inventory
    let token = head.while_obligation_locked(|token| {
        assert!(token.is_live());
        assert!(matches!(close.advance(request), O::Busy));
        assert!(token.is_live());
        token
    });
    assert!(matches!(close.advance(request), O::Advanced));
    assert!(!token.is_live(), "the SAME real head token was consumed");
    assert!(matches!(close.advance(request), O::Complete));
    let retired = close.into_retired_memory().unwrap();
    drop(token);
    drop(head);
    assert!(weak_root.upgrade().is_some());
    retired.reclaim();
    assert!(weak_root.upgrade().is_none());
}
