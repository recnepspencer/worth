//! Pure sorted-set, enclosure and AVL oracles, without execution/Source authority.
use super::*;
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Arc,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Enclosure(i64, i64);
impl Summary for Enclosure {
    fn merge(own: Self, children: [Option<Self>; 2]) -> Self {
        children
            .into_iter()
            .flatten()
            .fold(own, |a, b| Self(a.0.min(b.0), a.1.max(b.1)))
    }
}
struct Receiver(WatchSlot, Arc<AtomicUsize>);
impl Drop for Receiver {
    fn drop(&mut self) {
        self.1.fetch_add(1, AtomicOrdering::Relaxed);
    }
}
type Tree = PredicateTree<i64, Receiver, Enclosure>;

fn insert(tree: &mut Tree, slot: WatchSlot, probe: i64, drops: &Arc<AtomicUsize>) {
    let location = tree
        .locate(slot, &probe, &mut |a, b| Ok::<_, ()>(a.cmp(b)))
        .unwrap();
    tree.attach(
        slot,
        Record {
            probe,
            receiver: Receiver(slot, drops.clone()),
            own: Enclosure(probe, probe),
            aggregate: Enclosure(probe, probe),
            parent: None,
            children: [None; 2],
            extrema: [slot; 2],
            height: 1,
            _memory: None,
        },
        location,
    );
}

fn verify(tree: &Tree, expected: &BTreeMap<WatchSlot, i64>) {
    fn walk(
        tree: &Tree,
        id: Option<WatchSlot>,
        parent: Option<WatchSlot>,
        ordered: &mut Vec<(i64, WatchSlot)>,
        seen: &mut std::collections::BTreeSet<WatchSlot>,
    ) -> Option<(u8, Enclosure, [WatchSlot; 2])> {
        let id = id?;
        assert!(seen.insert(id), "no duplicate ownership or cycles");
        let n = &tree.records[&id];
        assert_eq!(n.parent, parent);
        assert_eq!(
            n.receiver.0, id,
            "immutable receiver follows its actual slot"
        );
        let left = walk(tree, n.children[0], Some(id), ordered, seen);
        ordered.push((n.probe, id));
        let right = walk(tree, n.children[1], Some(id), ordered, seen);
        let h = left.map_or(0, |x| x.0).max(right.map_or(0, |x| x.0)) + 1;
        assert!(left.map_or(0, |x| x.0).abs_diff(right.map_or(0, |x| x.0)) <= 1);
        // Independently enclose the actual reachable values, not stored summaries.
        let mut values = vec![n.probe];
        for child in [left, right].into_iter().flatten() {
            values.extend([child.1 .0, child.1 .1]);
        }
        let enclosure = Enclosure(*values.iter().min().unwrap(), *values.iter().max().unwrap());
        let extrema = [left.map_or(id, |x| x.2[0]), right.map_or(id, |x| x.2[1])];
        assert_eq!((n.height, n.aggregate, n.extrema), (h, enclosure, extrema));
        Some((h, enclosure, extrema))
    }
    let mut ordered = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    walk(tree, tree.root, None, &mut ordered, &mut seen);
    let mut oracle: Vec<_> = expected
        .iter()
        .map(|(slot, probe)| (*probe, *slot))
        .collect();
    oracle.sort();
    assert_eq!(ordered, oracle);
    assert_eq!(seen.len(), tree.len());
}

#[test]
fn duplicates_rotations_and_every_removal_preserve_actual_slot_probe_and_summary() {
    for order in [0, 1, 2] {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut tree = Tree::default();
        let mut oracle = BTreeMap::new();
        let ids: Vec<_> = (0..127)
            .map(|i| match order {
                0 => i,
                1 => 126 - i,
                _ => (i * 37) % 127,
            })
            .collect();
        for id in &ids {
            let slot = WatchSlot(*id);
            let probe = (*id % 23) as i64 - 11;
            insert(&mut tree, slot, probe, &drops);
            oracle.insert(slot, probe);
            verify(&tree, &oracle);
        }
        for (index, id) in ids.iter().rev().enumerate() {
            let retired = tree.remove(WatchSlot(*id)).unwrap();
            assert_eq!(
                retired.record.probe,
                oracle.remove(&WatchSlot(*id)).unwrap()
            );
            verify(&tree, &oracle);
            assert_eq!(drops.load(AtomicOrdering::Relaxed), index);
            drop(retired); // Removal returned the payload instead of dropping it.
            assert_eq!(drops.load(AtomicOrdering::Relaxed), index + 1);
        }
        assert!(tree.remove(WatchSlot(777)).is_none());
    }
}

#[test]
fn enclosing_prune_matches_independent_scan_and_unrelated_regions_stay_unvisited() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut tree = Tree::default();
    for id in 0..255 {
        insert(&mut tree, WatchSlot(id), id as i64, &drops);
    }
    for target in [-1, 0, 71, 254, 255] {
        let candidates = RefCell::new(Vec::new());
        let visited = RefCell::new(Vec::new());
        tree.walk(
            tree.root,
            &mut |_| Ok::<_, ()>(()),
            &mut |s, extrema| {
                assert_eq!((s.0, s.1), (*extrema[0], *extrema[1]));
                Ok(s.0 <= target && target <= s.1)
            },
            &mut |id, probe, _| {
                visited.borrow_mut().push(*probe);
                if *probe == target {
                    candidates.borrow_mut().push(id);
                }
                Ok(())
            },
        )
        .unwrap();
        let oracle: Vec<_> = (0..255)
            .filter(|id| *id as i64 == target)
            .map(WatchSlot)
            .collect();
        assert_eq!(*candidates.borrow(), oracle);
        assert!(
            visited.borrow().len() <= 8,
            "actual inverse descent stays logarithmic"
        );
    }
    drop(tree);
    assert_eq!(drops.load(AtomicOrdering::Relaxed), 255);
}

#[test]
fn comparison_refusal_precedes_mutation_and_removal_does_not_use_probe_comparisons() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut tree = Tree::default();
    insert(&mut tree, WatchSlot(1), 7, &drops);
    let location = tree.locate(WatchSlot(2), &9, &mut |_, _| Err::<Ordering, _>(23));
    assert_eq!(location, Err(23));
    verify(&tree, &BTreeMap::from([(WatchSlot(1), 7)]));
    assert_eq!(drops.load(AtomicOrdering::Relaxed), 0);
    // No comparison callback or control is accepted by requestless exact removal.
    drop(tree.remove(WatchSlot(1)).unwrap());
    assert_eq!(drops.load(AtomicOrdering::Relaxed), 1);
}
