//! Memory-Home parity: the volatile table runs the verified kernel's nested
//! lease law for parent -> child -> grandchild overlays.
use super::*;

const PARENT_ROOT: u64 = 10;
const CHILD_ROOT: u64 = 20;
const GRANDCHILD_ROOT: u64 = 30;

fn retain() -> RootPinDropPolicy {
    RootPinDropPolicy::RetainUntilChildrenDrain
}

#[test]
fn nested_grandchild_keeps_every_ancestor_root_until_it_drains() {
    let table = VolatileRootPinTable::default();
    let parent = table
        .pin(PARENT_ROOT, None, retain(), b"parent".to_vec())
        .unwrap();
    let child = table
        .pin(CHILD_ROOT, Some(parent), retain(), b"child".to_vec())
        .unwrap();
    let grandchild = table
        .pin(
            GRANDCHILD_ROOT,
            Some(child),
            retain(),
            b"grandchild".to_vec(),
        )
        .unwrap();
    assert_eq!(table.loan(grandchild).unwrap(), b"grandchild");

    // Active -> Expiring fences new parent loans; every ancestor stays protected.
    assert_eq!(table.expire(parent), Ok(RootPinState::Expiring));
    assert_eq!(table.loan(parent), Err(RootPinDenial::LeaseExpired));
    assert_eq!(table.expire(child), Ok(RootPinState::Expiring));
    for root in [PARENT_ROOT, CHILD_ROOT, GRANDCHILD_ROOT] {
        assert!(table.protects_root(root), "root {root} released early");
    }

    // The grandchild expires but keeps its loan: nothing above it can complete.
    assert_eq!(table.expire(grandchild), Ok(RootPinState::Expiring));
    assert!(table.protects_root(PARENT_ROOT));
    assert_eq!(table.payload(parent).as_deref(), Some(&b"parent"[..]));

    // Draining the last loan completes the whole chain bottom-up.
    table.release_loan(grandchild);
    for pin in [grandchild, child, parent] {
        assert_eq!(table.state(pin), Some(RootPinState::Expired));
    }
    for root in [PARENT_ROOT, CHILD_ROOT, GRANDCHILD_ROOT] {
        assert!(!table.protects_root(root), "root {root} leaked");
    }
}

#[test]
fn refuse_policy_blocks_parent_drop_while_a_child_lives() {
    let table = VolatileRootPinTable::default();
    let parent = table
        .pin(
            PARENT_ROOT,
            None,
            RootPinDropPolicy::RefuseIfPinnedByChild,
            Vec::new(),
        )
        .unwrap();
    let child = table
        .pin(CHILD_ROOT, Some(parent), retain(), Vec::new())
        .unwrap();
    assert_eq!(table.expire(parent), Err(RootPinDenial::PinnedByChild));
    assert_eq!(table.state(parent), Some(RootPinState::Active));
    assert_eq!(table.expire(child), Ok(RootPinState::Expired));
    assert_eq!(table.expire(parent), Ok(RootPinState::Expired));
    assert!(!table.protects_root(PARENT_ROOT));
}

#[test]
fn a_child_cannot_overlay_an_expiring_parent() {
    let table = VolatileRootPinTable::default();
    let parent = table.pin(PARENT_ROOT, None, retain(), Vec::new()).unwrap();
    let held = table.loan(parent).unwrap();
    assert_eq!(held, Vec::<u8>::new());
    assert_eq!(table.expire(parent), Ok(RootPinState::Expiring));
    assert_eq!(
        table.pin(CHILD_ROOT, Some(parent), retain(), Vec::new()),
        Err(RootPinDenial::ParentPinNotActive)
    );
    table.release_loan(parent);
    assert_eq!(table.state(parent), Some(RootPinState::Expired));
}
