use super::*;
use crate::{ExecutionRequest, ExecutionWorkCeiling};
#[test]
fn request_scope_reborrows_its_active_descendant_without_widening_policy() {
    let _guard = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1500, 100)).unwrap();
    let child = parent.child(request(1, 1200, 20)).unwrap();
    let execution = ExecutionRequest::leased(&parent);
    let (answer, _) = ExecutionWorkCeiling::new(Some(20))
        .run(&child, || {
            execution.in_scope(|lease| {
                let lease = lease.unwrap();
                assert_eq!(lease.policy(), child.policy());
                assert!(!lease.status().is_cancelled());
                7
            })
        })
        .unwrap();
    assert_eq!(answer, Ok(7));
}
#[test]
fn request_scope_never_substitutes_a_sibling_worker() {
    let _guard = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1500, 100)).unwrap();
    let first = parent.child(request(1, 1200, 20)).unwrap();
    let sibling = parent.child(request(1, 1100, 10)).unwrap();
    let original = ExecutionRequest::leased(&first);
    let (answer, _) = ExecutionWorkCeiling::new(Some(10))
        .run(&sibling, || {
            original.in_scope(|lease| {
                assert_eq!(lease.unwrap().policy(), first.policy());
                9
            })
        })
        .unwrap();
    assert_eq!(answer, Ok(9));
}
