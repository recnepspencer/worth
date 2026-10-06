use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::PartitionIdentity;

use super::{PartitionItemId, PartitionRoute, PartitionWork, SourceFactId};

mod checked;
mod retained;
use super::checked_scope::OwnedRetainedCharge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentDenial {
    UnknownItem(PartitionItemId),
    SelfEdge(PartitionItemId),
}

/// Connected components of an undirected coupling relation. A component's
/// identity is the least current item identity in that component.
#[derive(Default)]
pub struct ComponentPartitioner {
    facts: BTreeMap<PartitionItemId, SourceFactId>,
    adjacent: BTreeMap<PartitionItemId, BTreeSet<PartitionItemId>>,
    routes: BTreeMap<PartitionItemId, PartitionIdentity>,
    members: BTreeMap<PartitionIdentity, BTreeSet<PartitionItemId>>,
    edge_count: usize,
    retained_charge: OwnedRetainedCharge,
}

impl ComponentPartitioner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(&self, item: PartitionItemId) -> Option<PartitionRoute> {
        Some(PartitionRoute {
            partition: *self.routes.get(&item)?,
            source_fact: *self.facts.get(&item)?,
        })
    }

    pub fn members(&self, identity: PartitionIdentity) -> Option<&BTreeSet<PartitionItemId>> {
        self.members.get(&identity)
    }

    pub fn upsert_item(&mut self, item: PartitionItemId, fact: SourceFactId) -> PartitionWork {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        if let Some(previous) = self.facts.insert(item, fact) {
            return if previous == fact {
                PartitionWork::default()
            } else {
                PartitionWork {
                    members_visited: 1,
                    ..PartitionWork::default()
                }
            };
        }
        let identity = PartitionIdentity::new(item.0);
        self.adjacent.insert(item, BTreeSet::new());
        self.routes.insert(item, identity);
        self.members.insert(identity, BTreeSet::from([item]));
        PartitionWork {
            items_rerouted: 1,
            members_visited: 1,
            ..PartitionWork::default()
        }
    }

    pub fn add_edge(
        &mut self,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, ComponentDenial> {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        self.check_edge(a, b)?;
        if self.adjacent.get(&a).is_some_and(|set| set.contains(&b)) {
            return Ok(PartitionWork::default());
        }
        self.adjacent.get_mut(&a).unwrap().insert(b);
        self.adjacent.get_mut(&b).unwrap().insert(a);
        self.edge_count += 1;
        let a_identity = self.routes[&a];
        let b_identity = self.routes[&b];
        if a_identity == b_identity {
            return Ok(PartitionWork {
                edges_visited: 1,
                ..PartitionWork::default()
            });
        }
        let (keep, merge) = if a_identity < b_identity {
            (a_identity, b_identity)
        } else {
            (b_identity, a_identity)
        };
        let moved = self.members.remove(&merge).unwrap();
        let count = moved.len() as u64;
        for item in &moved {
            self.routes.insert(*item, keep);
        }
        self.members.get_mut(&keep).unwrap().extend(moved);
        Ok(PartitionWork {
            items_rerouted: count,
            members_visited: count,
            edges_visited: 1,
            ..PartitionWork::default()
        })
    }

    pub fn remove_edge(
        &mut self,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, ComponentDenial> {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        self.check_edge(a, b)?;
        if !self.adjacent.get_mut(&a).unwrap().remove(&b) {
            return Ok(PartitionWork::default());
        }
        self.adjacent.get_mut(&b).unwrap().remove(&a);
        self.edge_count -= 1;
        let island = self.routes[&a];
        Ok(self.rebuild_island(island))
    }

    pub fn remove_item(&mut self, item: PartitionItemId) -> PartitionWork {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        let Some(identity) = self.routes.remove(&item) else {
            return PartitionWork::default();
        };
        self.facts.remove(&item);
        let neighbors = self.adjacent.remove(&item).unwrap();
        self.edge_count -= neighbors.len();
        for neighbor in neighbors {
            self.adjacent.get_mut(&neighbor).unwrap().remove(&item);
        }
        self.members.get_mut(&identity).unwrap().remove(&item);
        let mut work = self.rebuild_island(identity);
        work.items_rerouted += 1;
        work.members_visited += 1;
        work
    }

    fn check_edge(&self, a: PartitionItemId, b: PartitionItemId) -> Result<(), ComponentDenial> {
        if a == b {
            return Err(ComponentDenial::SelfEdge(a));
        }
        if !self.facts.contains_key(&a) {
            return Err(ComponentDenial::UnknownItem(a));
        }
        if !self.facts.contains_key(&b) {
            return Err(ComponentDenial::UnknownItem(b));
        }
        Ok(())
    }

    fn rebuild_island(&mut self, identity: PartitionIdentity) -> PartitionWork {
        let old = self.members.remove(&identity).unwrap();
        if old.is_empty() {
            return PartitionWork::default();
        }
        let mut parent: BTreeMap<_, _> = old.iter().copied().map(|item| (item, item)).collect();
        let mut edges_visited = 0;
        for item in &old {
            for neighbor in &self.adjacent[item] {
                edges_visited += 1;
                if old.contains(neighbor) && item < neighbor {
                    union_least(&mut parent, *item, *neighbor);
                }
            }
        }
        let mut rebuilt: BTreeMap<PartitionIdentity, BTreeSet<PartitionItemId>> = BTreeMap::new();
        let mut rerouted = 0;
        for item in &old {
            let next = PartitionIdentity::new(find_least(&mut parent, *item).0);
            if self.routes.insert(*item, next) != Some(next) {
                rerouted += 1;
            }
            rebuilt.entry(next).or_default().insert(*item);
        }
        self.members.extend(rebuilt);
        PartitionWork {
            items_rerouted: rerouted,
            members_visited: old.len() as u64,
            edges_visited,
            ..PartitionWork::default()
        }
    }
}

fn find_least(
    parent: &mut BTreeMap<PartitionItemId, PartitionItemId>,
    mut item: PartitionItemId,
) -> PartitionItemId {
    let start = item;
    while parent[&item] != item {
        item = parent[&item];
    }
    let root = item;
    let mut cursor = start;
    while parent[&cursor] != root {
        let next = parent[&cursor];
        parent.insert(cursor, root);
        cursor = next;
    }
    item
}

fn union_least(
    parent: &mut BTreeMap<PartitionItemId, PartitionItemId>,
    a: PartitionItemId,
    b: PartitionItemId,
) {
    let a = find_least(parent, a);
    let b = find_least(parent, b);
    if a != b {
        let (least, other) = if a < b { (a, b) } else { (b, a) };
        parent.insert(other, least);
    }
}
