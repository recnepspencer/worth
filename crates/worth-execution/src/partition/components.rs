use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::PartitionIdentity;

use super::{PartitionItemId, PartitionWork};

mod checked;
mod discovery;
mod removal;
mod request;
mod retained;
use super::checked_scope::OwnedRetainedCharge;

/// One edit to an undirected coupling graph. Routing keys are not persistent names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentEdit {
    UpsertItem(PartitionItemId),
    RemoveItem(PartitionItemId),
    AddEdge(PartitionItemId, PartitionItemId),
    RemoveEdge(PartitionItemId, PartitionItemId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentDenial {
    UnknownItem(PartitionItemId),
    SelfEdge(PartitionItemId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ComponentStorageId(u64);

/// Connected components of an undirected coupling relation. A component's
/// identity is the least current item identity in that component.
#[derive(Default)]
pub struct ComponentPartitioner {
    adjacent: BTreeMap<PartitionItemId, BTreeSet<PartitionItemId>>,
    routes: BTreeMap<PartitionItemId, ComponentStorageId>,
    group_names: BTreeMap<ComponentStorageId, PartitionIdentity>,
    next_group: u64,
    forest: BTreeMap<PartitionItemId, BTreeSet<PartitionItemId>>,
    members: BTreeMap<PartitionIdentity, BTreeSet<PartitionItemId>>,
    edge_count: usize,
    retained_charge: OwnedRetainedCharge,
}

impl ComponentPartitioner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        self.routes
            .get(&item)
            .and_then(|group| self.group_names.get(group))
            .copied()
    }

    pub fn members(&self, identity: PartitionIdentity) -> Option<&BTreeSet<PartitionItemId>> {
        self.members.get(&identity)
    }

    pub fn upsert_item(&mut self, item: PartitionItemId) -> PartitionWork {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        if self.routes.contains_key(&item) {
            return PartitionWork::default();
        }
        let identity = PartitionIdentity::new(item.0);
        self.adjacent.insert(item, BTreeSet::new());
        let group = self.issue_group();
        self.routes.insert(item, group);
        self.group_names.insert(group, identity);
        self.forest.insert(item, BTreeSet::new());
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
        let a_identity = self.route(a).unwrap();
        let b_identity = self.route(b).unwrap();
        if a_identity == b_identity {
            return Ok(PartitionWork {
                edges_visited: 1,
                ..PartitionWork::default()
            });
        }
        let (keep, merge) = self.merge_sides(a_identity, b_identity);
        let keep_storage = self.routes[&PartitionItemId(keep.value())];
        let merge_storage = self.routes[&PartitionItemId(merge.value())];
        let mut kept = self.members.remove(&keep).unwrap();
        let moved = self.members.remove(&merge).unwrap();
        let count = moved.len() as u64;
        let identity = keep.min(merge);
        for member in &moved {
            self.routes.insert(*member, keep_storage);
        }
        kept.extend(moved);
        self.members.insert(identity, kept);
        self.group_names.remove(&merge_storage);
        self.group_names.insert(keep_storage, identity);
        self.forest.get_mut(&a).unwrap().insert(b);
        self.forest.get_mut(&b).unwrap().insert(a);
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
        if !self.adjacent[&a].contains(&b) {
            return Ok(PartitionWork::default());
        }
        let identity = self.route(a).unwrap();
        let plan = self
            .plan_removal(None, Some((a, b)), &mut discovery::Offline)
            .unwrap();
        self.check_group_capacity(plan.groups.len())
            .expect("offline component capacity");
        Ok(self.apply_removal(identity, None, Some((a, b)), plan))
    }

    pub fn remove_item(&mut self, item: PartitionItemId) -> PartitionWork {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        let Some(identity) = self.route(item) else {
            return PartitionWork::default();
        };
        let plan = self
            .plan_removal(Some(item), None, &mut discovery::Offline)
            .unwrap();
        self.check_group_capacity(plan.groups.len())
            .expect("offline component capacity");
        self.apply_removal(identity, Some(item), None, plan)
    }

    fn merge_sides(
        &self,
        a: PartitionIdentity,
        b: PartitionIdentity,
    ) -> (PartitionIdentity, PartitionIdentity) {
        if (self.members[&a].len(), std::cmp::Reverse(a))
            >= (self.members[&b].len(), std::cmp::Reverse(b))
        {
            (a, b)
        } else {
            (b, a)
        }
    }
    fn check_group_capacity(&self, count: usize) -> Option<()> {
        self.next_group
            .checked_add(u64::try_from(count).ok()?)
            .map(|_| ())
    }
    fn issue_group(&mut self) -> ComponentStorageId {
        let group = self.next_group;
        self.next_group = self
            .next_group
            .checked_add(1)
            .expect("component storage capacity admitted before effects");
        ComponentStorageId(group)
    }

    fn check_edge(&self, a: PartitionItemId, b: PartitionItemId) -> Result<(), ComponentDenial> {
        if a == b {
            return Err(ComponentDenial::SelfEdge(a));
        }
        if !self.routes.contains_key(&a) {
            return Err(ComponentDenial::UnknownItem(a));
        }
        if !self.routes.contains_key(&b) {
            return Err(ComponentDenial::UnknownItem(b));
        }
        Ok(())
    }
}
