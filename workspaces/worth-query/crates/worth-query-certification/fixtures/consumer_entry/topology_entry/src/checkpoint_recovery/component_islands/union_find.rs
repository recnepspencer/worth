//! Independent union-find over client-authored live edges, never Native groups.
use super::*;
pub(super) fn partitions(
    vertices: &BTreeMap<String, Member>,
    edges: &BTreeSet<(String, String)>,
) -> Vec<BTreeSet<Member>> {
    let indices: BTreeMap<_, _> = vertices
        .keys()
        .enumerate()
        .map(|(i, key)| (key, i))
        .collect();
    let mut parents: Vec<_> = (0..vertices.len()).collect();
    for (from, to) in edges {
        let left = root(&mut parents, indices[from]);
        let right = root(&mut parents, indices[to]);
        parents[right] = left;
    }
    let mut groups: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for (index, member) in vertices.values().enumerate() {
        groups
            .entry(root(&mut parents, index))
            .or_default()
            .insert(member.clone());
    }
    let mut groups: Vec<_> = groups.into_values().collect();
    groups.sort_by(|a, b| a.first().cmp(&b.first()));
    groups
}
fn root(parents: &mut [usize], mut node: usize) -> usize {
    while parents[node] != node {
        parents[node] = parents[parents[node]];
        node = parents[node];
    }
    node
}
pub(super) fn successors(rings: usize) -> BTreeSet<(String, String)> {
    (0..rings)
        .flat_map(|ring| {
            [("a", "b"), ("b", "c"), ("c", "a")].map(|(from, to)| (key(ring, from), key(ring, to)))
        })
        .collect()
}
