//! Actual head-cell contention must preserve the owner-issued obligation/root.
use super::*;
use crate::change_source::{RelationalMemoryCloseObservation as O, RelationalRuntimeHandle};
use crate::tests::support::runtime_with_test_schema;

#[test]
fn memory_terminal_close_head_contention_retires_actual_token_without_dropping_root() {
    let host = crate::test_execution_policy::serial_request();
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
    let locked = head.obligation.lock().unwrap();
    let token = locked.as_ref().unwrap().clone();
    assert!(token.lock().unwrap().is_live());
    assert!(matches!(close.advance(request), O::Busy));
    assert!(token.lock().unwrap().is_live());
    drop(locked);
    assert!(matches!(close.advance(request), O::Advanced));
    assert!(
        !token.lock().unwrap().is_live(),
        "the SAME real head token was consumed"
    );
    assert!(matches!(close.advance(request), O::Complete));
    let retired = close.into_retired_memory().unwrap();
    drop(token);
    drop(head);
    assert!(weak_root.upgrade().is_some());
    retired.reclaim();
    assert!(weak_root.upgrade().is_none());
}
