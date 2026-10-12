//! Observe pooled descendants after dispatch, before any marker overwrite.
use super::*;
use crate::ExecutionOwnerContext;
use std::sync::Mutex;
crate::needs_os_threads! {
#[test]
fn shipped_dispatch_releases_descendant_markers_before_another_dispatch() {
    let _exclusive = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(4, 2000, 100)).unwrap();
    assert!(lease.pool().broadcast(|_| ExecutionOwnerContext::is_active()).iter().all(|active| !active), "pool workers begin without an owner");
    let entered = Mutex::new(Vec::new());
    let gate = Overlap::new(4);
    let owner = ExecutionOwnerContext::enter();
    let failed: MapOutcome<(), u64> = map([0_u64; 4]).run_owned(Some(&lease), |_, _| {
        entered.lock().unwrap().push((std::thread::current().id(), ExecutionOwnerContext::is_active()));
        gate.wait();
        panic!("the shipped boundary contains the kernel panic");
    });
    assert!(matches!(failed, MapOutcome::Stopped { reason: MapStop::Failure { cause: MapKernelFailure::Panic, .. }, .. }));
    drop(owner);
    assert!(!ExecutionOwnerContext::is_active(), "caller custody restores its prior marker");
    let entered = entered.into_inner().unwrap();
    assert_eq!(entered.len(), 4);
    assert!(entered.iter().all(|(_, active)| *active), "owned descendants inherit their owner");
    let caller = std::thread::current().id();
    assert!(entered.iter().any(|(id, _)| *id != caller), "the shipped run used descendants");
    // This broadcasts on the authority's existing workers. Unlike another
    // native dispatch, it does not enter or overwrite an owner marker.
    let released = lease.pool().broadcast(|_| (std::thread::current().id(), ExecutionOwnerContext::is_active()));
    assert!(entered.iter().filter(|(id, _)| *id != caller).all(|(id, _)| released.iter().any(|(worker, _)| worker == id)), "the observation reaches the same descendant workers");
    assert!(released.iter().all(|(_, active)| !active), "shipped dispatch releases each descendant marker before any new dispatch");
}
}
