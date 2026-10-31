//! Independent interval scans protect keyed BVH refits, insertions and retirements.
use std::collections::{BTreeMap, BTreeSet};
use worth_query_consumer_values::{
    build_box_tree, insert_box_source, query_box_tree, remove_box_source, replace_box_source, Box2,
    Box2Coordinates, BoxEditStorage, BoxNode, BoxNodeLinks,
};

#[derive(Clone, Default)]
struct Tree {
    nodes: Vec<Option<BoxNode<usize, usize>>>,
    leaves: BTreeMap<usize, usize>,
}
impl BoxEditStorage for Tree {
    type Id = usize;
    type Source = usize;
    type Stop = &'static str;
    fn checkpoint(&mut self) -> Result<(), Self::Stop> {
        Ok(())
    }
    fn read(&mut self, id: usize) -> Result<BoxNode<usize, usize>, Self::Stop> {
        self.nodes.get(id).copied().flatten().ok_or("absent node")
    }
    fn append(&mut self, node: BoxNode<usize, usize>) -> Result<usize, Self::Stop> {
        let id = self.nodes.len();
        self.nodes.push(Some(node));
        if let BoxNodeLinks::Leaf { source } = node.links {
            assert!(self.leaves.insert(source, id).is_none());
        }
        Ok(id)
    }
    fn replace(&mut self, id: usize, node: BoxNode<usize, usize>) -> Result<(), Self::Stop> {
        self.read(id)?;
        self.nodes[id] = Some(node);
        Ok(())
    }
    fn remove(&mut self, id: usize) -> Result<(), Self::Stop> {
        let node = self.read(id)?;
        if let BoxNodeLinks::Leaf { source } = node.links {
            self.leaves.remove(&source);
        }
        self.nodes[id] = None;
        Ok(())
    }
    fn source_bounds(&mut self, source: usize) -> Result<Box2, Self::Stop> {
        let id = *self.leaves.get(&source).ok_or("absent Source")?;
        Ok(self.read(id)?.bounds)
    }
    fn invalid(&self) -> Self::Stop {
        "invalid box topology"
    }
}
fn bounds(lower: [i64; 2], upper: [i64; 2]) -> Box2Coordinates {
    Box2Coordinates { lower, upper }
}
fn parity(tree: &Tree, root: Option<usize>, values: &BTreeMap<usize, Box2Coordinates>) {
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    if let Some(root) = root {
        inspect_balance(tree, root, &mut seen, &mut ordered);
    }
    assert_eq!(ordered.len(), values.len());
    assert_eq!(
        ordered.into_iter().collect::<BTreeSet<_>>(),
        values.keys().copied().collect()
    );
    assert_eq!(seen.len(), tree.nodes.iter().flatten().count());
    for probe in [
        bounds([-100, -100], [100, 100]),
        bounds([0, 0], [0, 0]),
        bounds([-4, -3], [-2, 0]),
        bounds([8, 8], [10, 10]),
        bounds([90, 90], [91, 91]),
    ] {
        query_parity(tree, root, values, probe);
    }
    if let Some(root) = root {
        let actual = tree.nodes[root].unwrap().bounds.coordinates();
        for axis in 0..2 {
            assert_eq!(
                actual.lower[axis],
                values
                    .values()
                    .map(|value| value.lower[axis])
                    .min()
                    .unwrap()
            );
            assert_eq!(
                actual.upper[axis],
                values
                    .values()
                    .map(|value| value.upper[axis])
                    .max()
                    .unwrap()
            );
        }
    } else {
        assert!(values.is_empty());
    }
}

fn query_parity(
    tree: &Tree,
    root: Option<usize>,
    values: &BTreeMap<usize, Box2Coordinates>,
    probe: Box2Coordinates,
) {
    let mut result = Vec::new();
    query_box_tree(
        root,
        tree.nodes.iter().flatten().count(),
        Box2::new(probe).unwrap(),
        |id| tree.nodes.get(id).copied().flatten().ok_or("absent node"),
        |key| {
            result.push(key);
            Ok(())
        },
        |_| Ok(()),
        || "invalid query",
    )
    .unwrap();
    result.sort();
    let expected = values
        .iter()
        .filter(|(_, value)| {
            (0..2).all(|axis| {
                value.lower[axis] <= probe.upper[axis] && probe.lower[axis] <= value.upper[axis]
            })
        })
        .map(|(key, _)| *key)
        .collect::<Vec<_>>();
    assert_eq!(result, expected, "closed probe {probe:?}");
}

fn inspect_balance(
    tree: &Tree,
    id: usize,
    seen: &mut BTreeSet<usize>,
    ordered: &mut Vec<usize>,
) -> u8 {
    assert!(seen.insert(id), "one live visit per box node");
    let node = tree.nodes[id].unwrap();
    match node.links {
        BoxNodeLinks::Leaf { source } => {
            ordered.push(source);
            assert_eq!(node.last_source, source);
            assert_eq!(node.height, 1);
        }
        BoxNodeLinks::Branch { left, right } => {
            let a = inspect_balance(tree, left, seen, ordered);
            let b = inspect_balance(tree, right, seen, ordered);
            assert!(a.abs_diff(b) <= 1, "spatial routing stays balanced");
            assert_eq!(node.height, a.max(b) + 1);
            assert_eq!(node.last_source, *ordered.last().unwrap());
        }
    }
    node.height
}

#[test]
fn keyed_box_edits_match_scan_through_refit_insert_retire() {
    let authored = [
        (9, bounds([-4, -3], [-2, 0])),
        (2, bounds([0, 0], [0, 0])),
        (17, bounds([2, 4], [8, 9])),
        (4, bounds([-10, 8], [-9, 11])),
        (3, bounds([8, 8], [10, 10])),
        (22, bounds([0, 0], [0, 0])),
        (6, bounds([-1, -1], [1, 1])),
    ];
    cold_backing_accepts_keyed_edits(&authored);
    let mut tree = Tree::default();
    let mut root = None;
    let mut values = BTreeMap::new();
    for (key, raw) in authored {
        root = Some(insert_box_source(root, key, Box2::new(raw).unwrap(), &mut tree).unwrap());
        values.insert(key, raw);
        parity(&tree, root, &values);
    }
    let original = tree.clone();
    let original_root = root;
    let original_values = values.clone();
    let moved = bounds([90, 90], [91, 91]);
    root =
        Some(replace_box_source(root.unwrap(), 9, Box2::new(moved).unwrap(), &mut tree).unwrap());
    values.insert(9, moved);
    parity(&tree, root, &values);
    for key in [4, 3, 22, 2, 6, 9, 17] {
        root = remove_box_source(root.unwrap(), key, &mut tree).unwrap();
        values.remove(&key);
        parity(&tree, root, &values);
    }
    parity(&original, original_root, &original_values);
    assert!(tree.nodes.iter().all(Option::is_none));
}

fn cold_backing_accepts_keyed_edits(authored: &[(usize, Box2Coordinates)]) {
    let mut values = authored
        .iter()
        .enumerate()
        .map(|(index, (_, raw))| (index, *raw))
        .collect::<BTreeMap<_, _>>();
    let mut tree = Tree::default();
    let mut root = build_box_tree(
        values.len(),
        |key| Ok((key, Box2::new(values[&key]).unwrap())),
        &mut tree,
    )
    .unwrap();
    parity(&tree, root, &values);
    let moved = bounds([90, 90], [91, 91]);
    root =
        Some(replace_box_source(root.unwrap(), 0, Box2::new(moved).unwrap(), &mut tree).unwrap());
    values.insert(0, moved);
    parity(&tree, root, &values);
    let root = remove_box_source(root.unwrap(), 3, &mut tree).unwrap();
    values.remove(&3);
    // This key is outside the original cold span. Existing keys are not renumbered.
    let root = Some(insert_box_source(root, 91, Box2::new(moved).unwrap(), &mut tree).unwrap());
    values.insert(91, moved);
    parity(&tree, root, &values);
}

#[test]
fn keyed_box_relocation_reuses_live_cells_across_rank_changes_and_singletons() {
    let mut tree = Tree::default();
    let mut values = BTreeMap::new();
    let mut root = None;
    for source in 0..33 {
        let x = 4 * source as i64;
        let raw = bounds([x, 0], [x + 2, 0]);
        root = Some(insert_box_source(root, source, Box2::new(raw).unwrap(), &mut tree).unwrap());
        values.insert(source, raw);
    }
    let old = tree.clone();
    let old_root = root;
    let old_values = values.clone();
    let leaf = tree.leaves[&0];
    let slots = tree.nodes.len();
    let live = tree.nodes.iter().flatten().count();
    for (x, y) in [
        (-100, 0),
        (1_000, 0),
        (50, 1_000_000),
        (50, -1_000_000),
        (0, 0),
        (64, 0),
    ] {
        let raw = bounds([x, y], [x + 2, y]);
        root =
            Some(replace_box_source(root.unwrap(), 0, Box2::new(raw).unwrap(), &mut tree).unwrap());
        values.insert(0, raw);
        assert_eq!(
            tree.leaves[&0], leaf,
            "relocation keeps its actual still-live leaf coordinate"
        );
        assert_eq!(
            tree.nodes.len(),
            slots,
            "no new storage slot is appended for a bounds rekey"
        );
        assert_eq!(
            tree.nodes.iter().flatten().count(),
            live,
            "detached cells are not retired/reissued"
        );
        assert!(tree.nodes.iter().all(Option::is_some));
        parity(&tree, root, &values);
    }
    parity(&old, old_root, &old_values);
    let mut singleton = Tree::default();
    let raw = bounds([0, 0], [0, 0]);
    let root = insert_box_source(None, 7, Box2::new(raw).unwrap(), &mut singleton).unwrap();
    let moved = bounds([i64::MIN, i64::MAX], [i64::MIN, i64::MAX]);
    let after = replace_box_source(root, 7, Box2::new(moved).unwrap(), &mut singleton).unwrap();
    assert_eq!(after, root);
    assert_eq!(singleton.nodes.len(), 1);
    parity(&singleton, Some(after), &BTreeMap::from([(7, moved)]));
}
