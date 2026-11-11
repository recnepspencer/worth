//! The real cache keeps disarmed Exact/Historical payloads through reclamation.
use crate::change_source::{RelationalMemoryCloseObservation as O, RelationalRuntimeHandle};
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};
use worth_execution::ExecutionRequest;

#[test]
fn memory_terminal_close_cache_contention_preserves_exact_and_historical_allocations() {
    let host = crate::test_execution_policy::serial_request();
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
    let original_entries = cache.states.read().unwrap().len();
    assert!(original_entries >= 2);
    let mut close = RelationalRuntimeHandle::immutable(std::sync::Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    assert!(matches!(close.advance(request), O::Advanced));
    let locked = cache.states.write().unwrap();
    assert!(matches!(close.advance(request), O::Busy));
    assert_eq!(locked.len(), original_entries);
    assert!(locked
        .values()
        .all(|entry| entry.state.is_some() && entry.retirement.is_none()));
    drop(locked);
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
    let states = cache.states.read().unwrap();
    assert_eq!(
        states.len(),
        original_entries,
        "no shrink/rehash/payload accumulation"
    );
    assert!(states
        .values()
        .all(|entry| entry.state.is_none() && entry.retirement.is_some()));
    let key = states.first().unwrap().0.clone();
    drop(states);
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
