//! Reconnect forest cuts and stage only membership that leaves the largest piece.
use super::{
    discovery::{smaller_pieces, Discovery},
    *,
};

pub(super) struct Removal {
    pub(super) groups: Vec<BTreeSet<PartitionItemId>>,
    pub(super) forest_edges: Vec<(PartitionItemId, PartitionItemId)>,
    pub(super) work: PartitionWork,
}
impl ComponentPartitioner {
    pub(super) fn plan_removal<D: Discovery>(
        &self,
        item: Option<PartitionItemId>,
        edge: Option<(PartitionItemId, PartitionItemId)>,
        charge: &mut D,
    ) -> Result<Removal, D::Error> {
        let seeds: Vec<_> = match item {
            Some(item) => self.forest[&item].iter().copied().collect(),
            None => {
                let (a, b) = edge.expect("edge removal has endpoints");
                if self.forest[&a].contains(&b) {
                    vec![a, b]
                } else {
                    Vec::new()
                }
            }
        };
        let mut work = PartitionWork::default();
        let pieces = smaller_pieces(self, item, edge, &seeds, charge, &mut work)?;
        let mut part = BTreeMap::new();
        for (index, members) in pieces.iter().enumerate() {
            for &member in members {
                charge.member()?;
                work.members_visited += 1;
                part.insert(member, index + 1);
            }
        }
        let mut parents: Vec<_> = (0..seeds.len().max(1)).collect();
        let mut forest_edges = Vec::new();
        for (&source, &source_part) in &part {
            for &target in &self.adjacent[&source] {
                charge.edge()?;
                work.edges_visited += 1;
                if Some(target) == item
                    || edge.is_some_and(|(a, b)| {
                        (source == a && target == b) || (source == b && target == a)
                    })
                {
                    continue;
                }
                let target_part = part.get(&target).copied().unwrap_or(0);
                let a = root(&mut parents, source_part);
                let b = root(&mut parents, target_part);
                if a != b {
                    parents[a.max(b)] = a.min(b);
                    forest_edges.push((source, target));
                }
            }
        }
        let mut groups = BTreeMap::<usize, BTreeSet<PartitionItemId>>::new();
        for (index, members) in pieces.into_iter().enumerate() {
            let root = root(&mut parents, index + 1);
            if root != 0 {
                groups.entry(root).or_default().extend(members);
            }
        }
        Ok(Removal {
            groups: groups.into_values().collect(),
            forest_edges,
            work,
        })
    }

    /// The plan and publication checkpoint have succeeded. No control/refusal
    /// boundary follows until the graph and its routing witness agree again.
    pub(super) fn apply_removal(
        &mut self,
        identity: PartitionIdentity,
        item: Option<PartitionItemId>,
        edge: Option<(PartitionItemId, PartitionItemId)>,
        mut plan: Removal,
    ) -> PartitionWork {
        let storage = self.routes[&PartitionItemId(identity.value())];
        let mut retained = self.members.remove(&identity).expect("selected component");
        if let Some(item) = item {
            retained.remove(&item);
            self.routes.remove(&item);
            let adjacent = self.adjacent.remove(&item).expect("live item");
            self.edge_count -= adjacent.len();
            for neighbor in adjacent {
                self.adjacent.get_mut(&neighbor).unwrap().remove(&item);
            }
            let forest = self.forest.remove(&item).expect("live forest vertex");
            for neighbor in forest {
                self.forest.get_mut(&neighbor).unwrap().remove(&item);
            }
            plan.work.items_rerouted += 1;
            plan.work.members_visited += 1;
        } else if let Some((a, b)) = edge {
            self.adjacent.get_mut(&a).unwrap().remove(&b);
            self.adjacent.get_mut(&b).unwrap().remove(&a);
            self.forest.get_mut(&a).unwrap().remove(&b);
            self.forest.get_mut(&b).unwrap().remove(&a);
            self.edge_count -= 1;
        }
        for (a, b) in plan.forest_edges {
            self.forest.get_mut(&a).unwrap().insert(b);
            self.forest.get_mut(&b).unwrap().insert(a);
        }
        for group in plan.groups {
            let next = PartitionIdentity::new(group.first().unwrap().0);
            let physical = self.issue_group();
            for &member in &group {
                retained.remove(&member);
                self.routes.insert(member, physical);
            }
            plan.work.items_rerouted += group.len() as u64;
            plan.work.members_visited += group.len() as u64;
            self.group_names.insert(physical, next);
            self.members.insert(next, group);
        }
        if let Some(first) = retained.first() {
            let next = PartitionIdentity::new(first.0);
            self.group_names.insert(storage, next);
            self.members.insert(next, retained);
        } else {
            self.group_names.remove(&storage);
        }
        plan.work
    }
}
fn root(parents: &mut [usize], mut part: usize) -> usize {
    while parents[part] != part {
        parents[part] = parents[parents[part]];
        part = parents[part];
    }
    part
}
