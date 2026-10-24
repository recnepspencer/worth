use super::*;

#[test]
fn descent_names_the_lease_and_its_descendants_never_siblings_parents_or_other_authorities() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 4096, 100)).unwrap();
    let child = parent.child(request(1, 0, 100)).unwrap();
    let grandchild = child.controlled_child(CancellationToken::new(), None);
    let sibling = parent.child(request(1, 0, 100)).unwrap();
    let other_root = authority().request_lease(request(1, 4096, 100)).unwrap();
    let foreign = crate::ExecutionResourceLease::serial(request(1, 4096, 100)).unwrap();
    assert!(parent.descends_from(&parent));
    assert!(child.descends_from(&parent));
    assert!(grandchild.descends_from(&parent));
    assert!(grandchild.descends_from(&child));
    assert!(!parent.descends_from(&child), "a parent is not its child's descendant");
    assert!(!sibling.descends_from(&child));
    assert!(!child.descends_from(&sibling));
    assert!(!child.descends_from(&other_root));
    assert!(!foreign.descends_from(&parent));
    assert!(!parent.descends_from(&foreign));
}
