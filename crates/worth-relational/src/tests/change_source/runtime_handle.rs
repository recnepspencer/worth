use std::sync::{Arc, Mutex};

use crate::facade::change_source::RelationalRuntimeHandle;
use crate::tests::support::runtime_with_test_schema;

#[test]
fn a_shared_handle_describes_itself_while_its_runtime_is_held() {
    let runtime = runtime_with_test_schema();
    let expected = runtime.runtime_instance_id();
    let handle = RelationalRuntimeHandle::shared(Arc::new(Mutex::new(runtime)));

    let (id, debug) =
        handle.with_runtime(|_| (handle.runtime_instance_id(), format!("{handle:?}")));

    assert_eq!(id, expected);
    assert!(debug.contains(&format!("runtime_instance_id: {expected}")));
    assert!(debug.contains("shared"));
}

#[test]
fn owned_lock_observation_covers_clones_and_ends_with_its_guard() {
    let handle = RelationalRuntimeHandle::shared(Arc::new(Mutex::new(runtime_with_test_schema())));
    let clone = handle.clone();
    let count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let acquired = count.clone();
    let guard = handle.observe_locks_for_test(move || {
        acquired.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    clone.with_runtime(|_| ());
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    drop(guard);
    handle.with_runtime(|_| ());
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
}
