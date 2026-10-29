//! Stable local keys exercise value ordering independently of Source row positions.
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};
use worth_query_consumer_values::{
    build_prefix_tree, insert_prefix_source, prefix_order, query_prefix_tree, remove_prefix_source,
    PrefixBuildStorage, PrefixNode, PrefixOrder, PrefixQueryStorage,
};

#[derive(Clone, Default)]
struct Tree {
    values: BTreeMap<u64, String>,
    nodes: Vec<Option<PrefixNode<usize, u64>>>,
}
impl PrefixBuildStorage for Tree {
    type Id = usize;
    type Source = u64;
    type Stop = &'static str;
    fn checkpoint(&mut self) -> Result<(), Self::Stop> {
        Ok(())
    }
    fn compare_sources(&mut self, a: u64, b: u64) -> Result<Ordering, Self::Stop> {
        Ok(self
            .values
            .get(&a)
            .ok_or("absent source")?
            .cmp(self.values.get(&b).ok_or("absent source")?)
            .then(a.cmp(&b)))
    }
    fn append(&mut self, node: PrefixNode<usize, u64>) -> Result<usize, Self::Stop> {
        let id = self.nodes.len();
        self.nodes.push(Some(node));
        Ok(id)
    }
    fn read(&mut self, id: usize) -> Result<PrefixNode<usize, u64>, Self::Stop> {
        self.nodes.get(id).copied().flatten().ok_or("absent node")
    }
    fn replace(&mut self, id: usize, node: PrefixNode<usize, u64>) -> Result<(), Self::Stop> {
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
        "invalid prefix topology"
    }
}
struct Query<'a> {
    tree: &'a Tree,
    prefix: &'a str,
    result: Vec<u64>,
}
impl PrefixQueryStorage for Query<'_> {
    type Id = usize;
    type Source = u64;
    type Stop = &'static str;
    fn checkpoint(&mut self) -> Result<(), Self::Stop> {
        Ok(())
    }
    fn read(&mut self, id: usize) -> Result<PrefixNode<usize, u64>, Self::Stop> {
        self.tree
            .nodes
            .get(id)
            .copied()
            .flatten()
            .ok_or("absent node")
    }
    fn classify_source(&mut self, source: u64) -> Result<PrefixOrder, Self::Stop> {
        prefix_order(
            self.tree.values.get(&source).ok_or("absent source")?,
            self.prefix,
            |_| Ok(()),
        )
    }
    fn emit(&mut self, source: u64) -> Result<(), Self::Stop> {
        self.result.push(source);
        Ok(())
    }
    fn invalid(&self) -> Self::Stop {
        "invalid prefix query"
    }
}
fn parity(tree: &Tree, root: Option<usize>) {
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    inspect_balance(tree, root, &mut seen, &mut ordered);
    let mut expected_order = tree.values.keys().copied().collect::<Vec<_>>();
    expected_order.sort_by_key(|key| (&tree.values[key], *key));
    assert_eq!(ordered, expected_order);
    assert_eq!(seen.len(), tree.nodes.iter().flatten().count());
    for prefix in ["", "al", "alpha", "ál", "a\u{301}", "β", "missing"] {
        let mut query = Query {
            tree,
            prefix,
            result: Vec::new(),
        };
        query_prefix_tree(root, tree.nodes.iter().flatten().count(), &mut query).unwrap();
        query.result.sort();
        let expected = tree
            .values
            .iter()
            .filter(|(_, value)| value.starts_with(prefix))
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        assert_eq!(query.result, expected, "prefix {prefix:?}");
    }
}

fn inspect_balance(
    tree: &Tree,
    root: Option<usize>,
    seen: &mut BTreeSet<usize>,
    ordered: &mut Vec<u64>,
) -> u8 {
    let Some(id) = root else {
        return 0;
    };
    assert!(seen.insert(id), "one live visit per prefix node");
    let node = tree.nodes[id].unwrap();
    let left = inspect_balance(tree, node.left, seen, ordered);
    ordered.push(node.source);
    let right = inspect_balance(tree, node.right, seen, ordered);
    assert!(left.abs_diff(right) <= 1, "AVL balance survives an edit");
    assert_eq!(node.height, left.max(right) + 1);
    node.height
}

#[test]
fn keyed_prefix_edits_match_reference_through_rename_retire_reinsert() {
    // These are pure occurrence keys, not asserted Native identities or authority.
    let values = [
        (9, "alpha"),
        (2, "βeta"),
        (17, "alpha"),
        (4, "álamo"),
        (3, ""),
        (22, "alphabet"),
        (6, "a\u{301}lpha"),
    ];
    let mut tree = Tree {
        values: values
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect(),
        ..Tree::default()
    };
    let mut root = build_prefix_tree([9, 2, 17, 4, 3, 22, 6], &mut tree).unwrap();
    parity(&tree, root);
    let original = tree.clone();
    let original_root = root;
    root = remove_prefix_source(root, 9, &mut tree).unwrap();
    tree.values.insert(9, "alpine".into());
    root = Some(insert_prefix_source(root, 9, &mut tree).unwrap());
    root = remove_prefix_source(root, 17, &mut tree).unwrap();
    tree.values.remove(&17);
    tree.values.insert(91, "alpha".into());
    root = Some(insert_prefix_source(root, 91, &mut tree).unwrap());
    parity(&tree, root);
    parity(&original, original_root);
    for key in [4, 3, 22, 2, 6, 9, 91] {
        root = remove_prefix_source(root, key, &mut tree).unwrap();
        tree.values.remove(&key);
        parity(&tree, root);
    }
    assert!(root.is_none());
    assert!(tree.nodes.iter().all(Option::is_none));
}
