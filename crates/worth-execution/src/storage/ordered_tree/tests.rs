//! Independent set/order and finite-storage invariants for the promoted mechanic.
use super::*;
use std::collections::BTreeSet;

type Node = ExecutionOrderedTreeNode<usize, u32>;
#[derive(Default)]
struct Storage {
    nodes: Vec<Option<Node>>,
    visits: usize,
    stop_at: Option<usize>,
    reads: usize,
    appends: usize,
    comparisons: usize,
}
impl ExecutionOrderedTreeStorage for Storage {
    type Id = usize;
    type Source = u32;
    type Stop = &'static str;
    fn checkpoint(&mut self) -> Result<(), Self::Stop> {
        self.visits += 1;
        if self.stop_at.is_some_and(|at| self.visits >= at) {
            Err("stopped")
        } else {
            Ok(())
        }
    }
    fn compare_sources(&mut self, left: u32, right: u32) -> Result<Ordering, Self::Stop> {
        self.checkpoint()?;
        self.comparisons += 1;
        Ok(left.cmp(&right))
    }
    fn append(&mut self, node: Node) -> Result<usize, Self::Stop> {
        self.checkpoint()?;
        self.appends += 1;
        let id = self.nodes.len();
        self.nodes.push(Some(node));
        Ok(id)
    }
    fn read(&mut self, id: usize) -> Result<Node, Self::Stop> {
        self.checkpoint()?;
        self.reads += 1;
        self.nodes.get(id).copied().flatten().ok_or("missing node")
    }
    fn replace(&mut self, id: usize, node: Node) -> Result<(), Self::Stop> {
        self.read(id)?;
        self.nodes[id] = Some(node);
        Ok(())
    }
    fn remove(&mut self, id: usize) -> Result<(), Self::Stop> {
        self.read(id)?;
        self.nodes[id] = None;
        Ok(())
    }
    fn invalid(&self) -> Self::Stop {
        "invalid"
    }
}

fn verify(storage: &Storage, root: Option<usize>, expected: &BTreeSet<u32>) {
    fn visit(
        storage: &Storage,
        id: usize,
        seen: &mut BTreeSet<usize>,
        order: &mut Vec<u32>,
    ) -> (u8, u32, u32) {
        assert!(seen.insert(id), "each live node occurs once");
        let node = storage.nodes[id].expect("reachable nodes are live");
        let left = node.left.map(|id| visit(storage, id, seen, order));
        order.push(node.source);
        let right = node.right.map(|id| visit(storage, id, seen, order));
        let lh = left.map_or(0, |value| value.0);
        let rh = right.map_or(0, |value| value.0);
        assert!(lh.abs_diff(rh) <= 1);
        assert_eq!(node.height, 1 + lh.max(rh));
        let lower = left.map_or(node.source, |value| value.1);
        let upper = right.map_or(node.source, |value| value.2);
        assert_eq!(node.bounds, ExecutionOrderedTreeBounds { lower, upper });
        (node.height, lower, upper)
    }
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    if let Some(root) = root {
        visit(storage, root, &mut seen, &mut order);
    }
    assert_eq!(order, expected.iter().copied().collect::<Vec<_>>());
    assert_eq!(
        seen.len(),
        storage.nodes.iter().flatten().count(),
        "no disconnected live storage"
    );
}

#[test]
fn rotations_extrema_and_repeated_removal_match_an_independent_set() {
    let mut storage = Storage::default();
    let mut root = None;
    let mut expected = BTreeSet::new();
    // Monotone insertion forces rotations; mixed removals include two-child roots.
    for item in 0..257 {
        root = Some(insert_ordered_tree(root, item, &mut storage).unwrap());
        expected.insert(item);
        verify(&storage, root, &expected);
    }
    let mut seed = 919u32;
    for _ in 0..1024 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let item = seed % 257;
        if expected.remove(&item) {
            root = remove_ordered_tree(root, item, &mut storage).unwrap();
        } else {
            root = Some(insert_ordered_tree(root, item, &mut storage).unwrap());
            expected.insert(item);
        }
        verify(&storage, root, &expected);
    }
    for item in expected.clone() {
        root = remove_ordered_tree(root, item, &mut storage).unwrap();
        expected.remove(&item);
        verify(&storage, root, &expected);
    }
    assert!(root.is_none());
}

#[test]
fn original_storage_stop_propagates_without_further_access() {
    let mut storage = Storage::default();
    let root = build_ordered_tree(0..32, &mut storage).unwrap();
    let stop = storage.visits + 3;
    storage.stop_at = Some(stop);
    assert_eq!(insert_ordered_tree(root, 99, &mut storage), Err("stopped"));
    assert_eq!(storage.visits, stop);
}

#[test]
fn sorted_construction_has_exact_linear_storage_work_and_supports_later_edits() {
    for count in [0u32, 1, 2, 3, 4, 17, 160, 257, 1024] {
        let mut storage = Storage::default();
        let mut root =
            build_sorted_ordered_tree(count as usize, (0..count).map(Ok), &mut storage).unwrap();
        let mut expected = (0..count).collect();
        verify(&storage, root, &expected);
        assert_eq!(storage.reads, 0);
        assert_eq!(storage.appends, count as usize);
        assert_eq!(storage.comparisons, count.saturating_sub(1) as usize);
        root = Some(insert_ordered_tree(root, count + 1, &mut storage).unwrap());
        expected.insert(count + 1);
        verify(&storage, root, &expected);
        for item in (0..count).step_by(3) {
            root = remove_ordered_tree(root, item, &mut storage).unwrap();
            expected.remove(&item);
        }
        verify(&storage, root, &expected);
    }
}

#[test]
fn sorted_construction_refuses_wrong_count_order_and_original_stops() {
    for (count, values) in [
        (0, vec![1]),
        (3, vec![1, 2]),
        (2, vec![1, 2, 3]),
        (3, vec![1, 1, 2]),
        (3, vec![1, 3, 2]),
    ] {
        assert_eq!(
            build_sorted_ordered_tree(count, values.into_iter().map(Ok), &mut Storage::default()),
            Err("invalid")
        );
    }
    assert_eq!(
        build_sorted_ordered_tree(
            3,
            [Ok(1), Err("input stop"), Ok(3)],
            &mut Storage::default()
        ),
        Err("input stop")
    );
    let mut funded = Storage::default();
    build_sorted_ordered_tree(17, (0..17).map(Ok), &mut funded).unwrap();
    for stop in 1..=funded.visits {
        let mut storage = Storage {
            stop_at: Some(stop),
            ..Default::default()
        };
        assert_eq!(
            build_sorted_ordered_tree(17, (0..17).map(Ok), &mut storage),
            Err("stopped")
        );
        assert_eq!(
            storage.visits, stop,
            "no storage call follows original stop"
        );
    }
}
