use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use super::PersistentVector;

static NESTED_ITEM_CLONES: AtomicUsize = AtomicUsize::new(0);
static CLONE_PROBE: Mutex<()> = Mutex::new(());

#[derive(Debug, Eq, PartialEq)]
struct Counted(u64);

impl Clone for Counted {
    fn clone(&self) -> Self {
        NESTED_ITEM_CLONES.fetch_add(1, Ordering::Relaxed);
        Self(self.0)
    }
}

#[derive(Clone, Copy, Debug)]
enum NestedPayloadPosture {
    OriginalExclusiveBase,
    AppendedTail,
}

fn nested_payload(len: usize) -> Vec<Counted> {
    (0..len).map(|value| Counted(value as u64)).collect()
}

#[test]
fn reservation_stays_on_the_flat_ordinary_lane() {
    let mut ordinary = PersistentVector::<u64, 64>::new();
    ordinary.reserve_exclusive(4_096);
    assert!(ordinary
        .exclusive_capacity()
        .is_some_and(|capacity| capacity >= 4_096));
    ordinary.extend(0..4_096);

    let mut fork = ordinary.fork_persistent();
    assert!(ordinary.shares_storage_with(&fork));
    fork.reserve_exclusive(65_536);
    assert!(
        ordinary.shares_storage_with(&fork),
        "reservation must not flatten a genuinely shared fork"
    );

    fork.push_back(4_096);
    assert_eq!(ordinary.len(), 4_096);
    assert_eq!(fork.len(), 4_097);
    assert_eq!(fork.get(4_096), Some(&4_096));
}

#[test]
fn fork_push_does_not_clone_an_inherited_nested_payload() {
    let _probe = CLONE_PROBE
        .lock()
        .expect("clone probe lock remains healthy");
    for posture in [
        NestedPayloadPosture::OriginalExclusiveBase,
        NestedPayloadPosture::AppendedTail,
    ] {
        for inherited_len in [64_usize, 4_096, 65_536] {
            let (source, mut destination) = match posture {
                NestedPayloadPosture::OriginalExclusiveBase => {
                    let mut source: PersistentVector<Vec<Counted>> = PersistentVector::new();
                    source.push_back(nested_payload(inherited_len));
                    NESTED_ITEM_CLONES.store(0, Ordering::Relaxed);
                    let destination = source.fork_persistent();
                    (source, destination)
                }
                NestedPayloadPosture::AppendedTail => {
                    let mut empty: PersistentVector<Vec<Counted>> = PersistentVector::new();
                    let mut source = empty.fork_persistent();
                    source.push_back(nested_payload(inherited_len));
                    NESTED_ITEM_CLONES.store(0, Ordering::Relaxed);
                    let destination = source.fork_persistent();
                    (source, destination)
                }
            };
            assert_eq!(
                NESTED_ITEM_CLONES.load(Ordering::Relaxed),
                0,
                "fork copied nested payload in posture {posture:?}, length {inherited_len}"
            );
            assert!(source.shares_storage_with(&destination));

            NESTED_ITEM_CLONES.store(0, Ordering::Relaxed);
            destination.push_back(vec![Counted(u64::MAX)]);

            assert_eq!(
                NESTED_ITEM_CLONES.load(Ordering::Relaxed),
                0,
                "posture {posture:?}, inherited nested payload length {inherited_len}"
            );
            assert_eq!(source.len(), 1);
            assert_eq!(source[0].len(), inherited_len);
            assert_eq!(destination.len(), 2);
            assert_eq!(destination[0].len(), inherited_len);
            assert_eq!(destination[1], vec![Counted(u64::MAX)]);
        }
    }
}

#[test]
fn fork_shared_mutation_and_reconstruction_match_a_flat_model() {
    let mut source = (0_u64..6)
        .map(|value| vec![value])
        .collect::<PersistentVector<_, 4>>();
    let source_model = (0_u64..6).map(|value| vec![value]).collect::<Vec<_>>();
    let mut fork = source.fork_persistent();
    let mut model = source_model.clone();

    assert_eq!(fork.pop_back(), model.pop());
    fork.push_back(vec![50]);
    model.push(vec![50]);
    fork[1].push(10);
    model[1].push(10);
    fork.push_back(vec![60]);
    model.push(vec![60]);
    fork.push_back(vec![70]);
    model.push(vec![70]);
    fork.push_back(vec![80]);
    model.push(vec![80]);

    let sibling_model = model.clone();
    let sibling = fork.fork_persistent();
    assert!(fork.shares_storage_with(&sibling));
    assert_eq!(fork.pop_back(), model.pop());
    fork[2] = vec![20, 21];
    model[2] = vec![20, 21];
    assert_eq!(fork.iter().cloned().collect::<Vec<_>>(), model);
    assert_eq!(
        serde_json::to_vec(&fork).unwrap(),
        serde_json::to_vec(&model).unwrap()
    );
    assert_eq!(source.iter().cloned().collect::<Vec<_>>(), source_model);
    assert_eq!(sibling.iter().cloned().collect::<Vec<_>>(), sibling_model);
    assert_ne!(sibling, fork);

    let mut drained = sibling.clone();
    let mut drained_model = sibling_model.clone();
    while !drained_model.is_empty() {
        assert_eq!(drained.pop_back(), drained_model.pop());
    }
    assert!(drained.is_empty());
    assert_eq!(drained.pop_back(), None);
    let refill = (100_u64..108).map(|value| vec![value]).collect::<Vec<_>>();
    drained.extend(refill.clone());
    assert_eq!(drained.iter().cloned().collect::<Vec<_>>(), refill);
    assert_ne!(
        drained, sibling,
        "refill must not resurrect stale base values"
    );

    let operational = fork.operational_clone();
    assert_eq!(operational, fork);
    assert!(!operational.shares_storage_with(&fork));

    fork.insert(1, vec![99]);
    model.insert(1, vec![99]);
    for value in fork.iter_mut() {
        value.push(100);
    }
    for value in &mut model {
        value.push(100);
    }
    assert_eq!(fork.iter().cloned().collect::<Vec<_>>(), model);

    fork.clear();
    model.clear();
    fork.extend([vec![7], vec![8]]);
    model.extend([vec![7], vec![8]]);
    assert_eq!(fork.iter().cloned().collect::<Vec<_>>(), model);
    assert!(fork.exclusive_capacity().is_some());
}
