//! Unsigned-domain edits use the same tree, checked against an independent census.
use super::*;
use crate::{segment_intersects_box, Box2Coordinates, ModelLocalMetre};
use std::collections::BTreeMap;
type Bounds = IntegerBox2<ModelLocalMetre>;
type Source = (u64, u32);
type Node = BoxNode<usize, Source, ModelLocalMetre>;
#[derive(Default)]
struct Tree {
    nodes: Vec<Option<Node>>,
    rows: BTreeMap<Source, Box2Coordinates<u64>>,
}
impl BoxEditStorage<ModelLocalMetre> for Tree {
    type Id = usize;
    type Source = Source;
    type Stop = &'static str;
    fn checkpoint(&mut self) -> Result<(), Self::Stop> {
        Ok(())
    }
    fn read(&mut self, id: usize) -> Result<Node, Self::Stop> {
        self.nodes.get(id).copied().flatten().ok_or("missing node")
    }
    fn append(&mut self, node: Node) -> Result<usize, Self::Stop> {
        self.nodes.push(Some(node));
        Ok(self.nodes.len() - 1)
    }
    fn replace(&mut self, id: usize, node: Node) -> Result<(), Self::Stop> {
        self.nodes[id] = Some(node);
        Ok(())
    }
    fn remove(&mut self, id: usize) -> Result<(), Self::Stop> {
        self.nodes[id] = None;
        Ok(())
    }
    fn source_bounds(&mut self, source: Source) -> Result<Bounds, Self::Stop> {
        Ok(Bounds::new(self.rows[&source]).unwrap())
    }
    fn invalid(&self) -> Self::Stop {
        "invalid topology"
    }
}
fn raw(lower: [u64; 2], upper: [u64; 2]) -> Box2Coordinates<u64> {
    Box2Coordinates { lower, upper }
}
fn verify(tree: &mut Tree, root: Option<usize>) {
    let m = u64::MAX;
    for probe in [
        raw([0; 2], [m; 2]),
        raw([0; 2], [0; 2]),
        raw([m; 2], [m; 2]),
        raw([0, 5], [100, 7]),
        raw([m - 100, 5], [m, 7]),
        raw([m / 2, 0], [m / 2 + 10, m]),
    ] {
        let expected: Vec<_> = tree
            .rows
            .iter()
            .filter(|(_, r)| {
                (0..2).all(|axis| {
                    r.lower[axis] <= probe.upper[axis] && probe.lower[axis] <= r.upper[axis]
                })
            })
            .map(|(key, _)| *key)
            .collect();
        let count = tree.nodes.iter().filter(|n| n.is_some()).count();
        let mut actual = Vec::new();
        query_box_tree(
            root,
            count,
            Bounds::new(probe).unwrap(),
            |id| tree.read(id),
            |key| {
                actual.push(key);
                Ok(())
            },
            |_| Ok(()),
            || "invalid query",
        )
        .unwrap();
        actual.sort();
        assert_eq!(actual, expected, "{probe:?}");
    }
}
#[test]
fn unsigned_cold_and_warm_edits_preserve_full_width_bounds_and_generation_ties() {
    let m = u64::MAX;
    let mut tree = Tree::default();
    for i in 0..64u64 {
        let x = if i % 2 == 0 { i * 3 } else { m - i * 3 };
        tree.rows.insert((i, 1), raw([x, 5], [x + 2, 7]));
    }
    tree.rows.insert((90, 1), raw([0; 2], [0; 2]));
    tree.rows.insert((90, 2), raw([m; 2], [m; 2]));
    tree.rows.insert((91, 1), raw([m; 2], [m; 2]));
    let source: Vec<_> = tree
        .rows
        .iter()
        .map(|(key, b)| (*key, Bounds::new(*b).unwrap()))
        .collect();
    let mut root = build_box_tree(source.len(), |i| Ok(source[i]), &mut tree).unwrap();
    verify(&mut tree, root);
    for i in 0..12u64 {
        let moved = raw([m / 2 + i * 3, 6], [m / 2 + i * 3 + 1, 6]);
        root = Some(
            replace_box_source(
                root.unwrap(),
                (i, 1),
                Bounds::new(moved).unwrap(),
                &mut tree,
            )
            .unwrap(),
        );
        tree.rows.insert((i, 1), moved);
        verify(&mut tree, root);
    }
    for key in [(90, 2), (2, 1), (7, 1), (91, 1)] {
        root = remove_box_source(root.unwrap(), key, &mut tree).unwrap();
        tree.rows.remove(&key);
        verify(&mut tree, root);
    }
    let added = raw([m - 1, m - 1], [m, m]);
    root = Some(insert_box_source(root, (90, 3), Bounds::new(added).unwrap(), &mut tree).unwrap());
    tree.rows.insert((90, 3), added);
    verify(&mut tree, root);
}
#[test]
fn unsigned_closed_contact_handles_extrema_cancellation_and_degenerate_boundaries() {
    let m = u64::MAX;
    let at = Bounds::new(raw([m - 1, m - 2], [m - 1, m - 2])).unwrap();
    assert!(!segment_intersects_box([0, 0], [m, m - 1], at));
    assert!(segment_intersects_box(
        [0, 0],
        [m, m],
        Bounds::new(raw([m / 2, m / 2], [m / 2, m / 2])).unwrap()
    ));
    assert!(segment_intersects_box(
        [m, m],
        [m, m],
        Bounds::new(raw([m - 1, m - 1], [m, m])).unwrap()
    ));
    assert!(!segment_intersects_box(
        [0, m],
        [m, m],
        Bounds::new(raw([0, 0], [m, m - 1])).unwrap()
    ));
    assert_eq!(
        Bounds::new(raw([1, 0], [0, m])),
        Err(crate::BoundsDenial::ReversedAxis { axis: 0 })
    );
}
