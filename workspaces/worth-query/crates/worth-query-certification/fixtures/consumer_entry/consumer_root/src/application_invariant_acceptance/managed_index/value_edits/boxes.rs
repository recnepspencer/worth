//! Independent interval scans protect keyed BVH refits, insertions and retirements.
use std::collections::{BTreeMap, BTreeSet};
use worth_query_consumer_values::{
    build_box_tree, insert_box_source, query_box_tree, remove_box_source, replace_box_source, Box2,
    Box2Coordinates, BoxEditStorage, BoxNode, BoxNodeLinks,
};

#[derive(Clone, Default)]
struct Tree {
    nodes: Vec<Option<BoxNode<usize, usize>>>,
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
        Ok(id)
    }
    fn replace(&mut self, id: usize, node: BoxNode<usize, usize>) -> Result<(), Self::Stop> {
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
    assert_eq!(ordered, values.keys().copied().collect::<Vec<_>>());
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
            assert!(a.abs_diff(b) <= 1, "BVH identity routing stays balanced");
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
    replace_box_source(root.unwrap(), 9, Box2::new(moved).unwrap(), &mut tree).unwrap();
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
    let root = build_box_tree(
        values.len(),
        |key| Ok(Box2::new(values[&key]).unwrap()),
        |node| tree.append(node),
        |_| Ok(()),
    )
    .unwrap();
    parity(&tree, root, &values);
    let moved = bounds([90, 90], [91, 91]);
    replace_box_source(root.unwrap(), 0, Box2::new(moved).unwrap(), &mut tree).unwrap();
    values.insert(0, moved);
    parity(&tree, root, &values);
    let root = remove_box_source(root.unwrap(), 3, &mut tree).unwrap();
    values.remove(&3);
    // This key is outside the original cold span. Existing keys are not renumbered.
    let root = Some(insert_box_source(root, 91, Box2::new(moved).unwrap(), &mut tree).unwrap());
    values.insert(91, moved);
    parity(&tree, root, &values);
}
