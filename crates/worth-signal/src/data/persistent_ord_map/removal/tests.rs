use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug)]
struct Payload(Arc<AtomicUsize>);
impl Clone for Payload {
    fn clone(&self) -> Self {
        self.0.fetch_add(1, Ordering::Relaxed);
        Self(self.0.clone())
    }
}

#[test]
fn discard_retirement_never_clones_removed_payload() {
    let clones = Arc::new(AtomicUsize::new(0));
    let mut source = PersistentOrdMap::new();
    for key in 0..64 {
        source.insert(key, Payload(clones.clone()));
    }
    let mut fork = source.fork_persistent();
    fork.insert(100, Payload(clones.clone()));
    let sibling = fork.fork_persistent();
    clones.store(0, Ordering::Relaxed);
    // Exercise adjacent base retirements, an overlay-only key, and absence.
    for key in [20, 21, 19, 100] {
        assert!(fork.remove_discard(&key));
        assert!(!fork.remove_discard(&key));
        assert!(fork.get(&key).is_none());
        assert!(sibling.get(&key).is_some());
    }
    assert!(!fork.remove_discard(&101));
    assert_eq!(clones.load(Ordering::Relaxed), 0);
    assert_eq!(fork.len(), 61);
    fork.insert(20, Payload(clones.clone()));
    assert!(fork.remove_discard(&20));
    assert_eq!(clones.load(Ordering::Relaxed), 0);
    assert_eq!(source.len(), 64);
    let mut exclusive = PersistentOrdMap::new();
    exclusive.insert(0, Payload(clones.clone()));
    assert!(exclusive.remove_discard(&0));
    assert!(exclusive.is_empty());
    assert_eq!(clones.load(Ordering::Relaxed), 0);
}
