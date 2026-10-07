use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::PartitionIdentity;

use super::{PartitionItemId, PartitionWork};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedItem {
    pub item: PartitionItemId,
    pub weight: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeightedEdge {
    pub a: PartitionItemId,
    pub b: PartitionItemId,
    pub weight: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BisectionDenial {
    ZeroWeight,
    WeightOverflow,
    PathExhausted,
    UnknownItem(PartitionItemId),
    SelfEdge(PartitionItemId),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BisectionQuality {
    pub cut_weight: u64,
    pub maximum_cut_imbalance: u64,
    pub maximum_leaf_weight: u64,
    pub cut_count: u64,
}

mod checked;
mod cut_tree;
mod retained;
#[cfg(test)]
mod tests;
use super::checked_scope::OwnedRetainedCharge;
use cut_tree::{
    collect_interfaces, collect_leaves, lightest_leaf, ordered_pair, summarize_quality,
    update_path, CutTree,
};

/// Retained recursive bisection. A leaf is identified by its root-to-leaf cut
/// path; only an overloaded leaf or an ancestor outside tolerance is re-cut.
pub struct Bisection {
    items: BTreeMap<PartitionItemId, WeightedItem>,
    edges: BTreeMap<(PartitionItemId, PartitionItemId), u64>,
    incident: BTreeMap<PartitionItemId, BTreeSet<(PartitionItemId, PartitionItemId)>>,
    total_edge_weight: u64,
    tree: Option<CutTree>,
    routes: BTreeMap<PartitionItemId, PartitionIdentity>,
    max_leaf_weight: u64,
    tolerance: u64,
    retained_charge: OwnedRetainedCharge,
}

impl Bisection {
    pub fn new(max_leaf_weight: u64, tolerance: u64) -> Result<Self, BisectionDenial> {
        if max_leaf_weight == 0 {
            return Err(BisectionDenial::ZeroWeight);
        }
        Ok(Self {
            items: BTreeMap::new(),
            edges: BTreeMap::new(),
            incident: BTreeMap::new(),
            total_edge_weight: 0,
            tree: None,
            routes: BTreeMap::new(),
            max_leaf_weight,
            tolerance,
            retained_charge: OwnedRetainedCharge::default(),
        })
    }

    pub fn route(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        self.routes.get(&item).copied()
    }

    pub fn upsert_item(&mut self, item: WeightedItem) -> Result<PartitionWork, BisectionDenial> {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        if item.weight == 0 {
            return Err(BisectionDenial::ZeroWeight);
        }
        if self.items.get(&item.item) == Some(&item) {
            return Ok(PartitionWork::default());
        }
        let previous = self.items.get(&item.item).copied();
        let total = self
            .tree
            .as_ref()
            .map_or(0, |tree| tree.weight)
            .checked_sub(previous.map_or(0, |old| old.weight))
            .unwrap()
            .checked_add(item.weight)
            .ok_or(BisectionDenial::WeightOverflow)?;
        self.items.insert(item.item, item);
        let mut work = PartitionWork {
            members_visited: 1,
            ..PartitionWork::default()
        };
        if self.tree.is_none() {
            self.incident.entry(item.item).or_default();
            self.tree = Some(CutTree {
                path: 1,
                members: BTreeSet::from([item.item]),
                weight: total,
                children: None,
            });
            self.routes.insert(item.item, PartitionIdentity::new(1));
            work.items_rerouted = 1;
            return Ok(work);
        }
        let target = self.routes.get(&item.item).copied().unwrap_or_else(|| {
            let tree = self.tree.as_ref().unwrap();
            lightest_leaf(tree).0
        });
        let tree = self.tree.as_mut().unwrap();
        let update = update_path(
            tree,
            target.value(),
            item.item,
            previous.map_or(0, |old| old.weight),
            item.weight,
            &self.items,
            &self.edges,
            &self.incident,
            self.max_leaf_weight,
            self.tolerance,
            &mut self.routes,
            &mut work,
        );
        if let Err(denial) = update {
            if let Some(previous) = previous {
                self.items.insert(item.item, previous);
            } else {
                self.items.remove(&item.item);
            }
            return Err(denial);
        }
        self.incident.entry(item.item).or_default();
        Ok(work)
    }

    pub fn remove_item(&mut self, item: PartitionItemId) -> Result<PartitionWork, BisectionDenial> {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        let Some(previous) = self.items.get(&item).copied() else {
            return Ok(PartitionWork::default());
        };
        let target = self.routes[&item].value();
        let mut work = PartitionWork {
            items_rerouted: 1,
            members_visited: 1,
            ..PartitionWork::default()
        };
        if self.items.len() > 1 {
            update_path(
                self.tree.as_mut().unwrap(),
                target,
                item,
                previous.weight,
                0,
                &self.items,
                &self.edges,
                &self.incident,
                self.max_leaf_weight,
                self.tolerance,
                &mut self.routes,
                &mut work,
            )?;
        } else {
            self.tree = None;
        }
        self.items.remove(&item);
        self.routes.remove(&item);
        let incident = self.incident.remove(&item).unwrap();
        work.edges_visited += incident.len() as u64;
        for pair in incident {
            self.total_edge_weight -= self.edges.remove(&pair).unwrap();
            let other = if pair.0 == item { pair.1 } else { pair.0 };
            self.incident.get_mut(&other).unwrap().remove(&pair);
        }
        Ok(work)
    }

    pub fn upsert_edge(&mut self, edge: WeightedEdge) -> Result<PartitionWork, BisectionDenial> {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        if edge.a == edge.b {
            return Err(BisectionDenial::SelfEdge(edge.a));
        }
        if edge.weight == 0 {
            return Err(BisectionDenial::ZeroWeight);
        }
        for endpoint in [edge.a, edge.b] {
            if !self.items.contains_key(&endpoint) {
                return Err(BisectionDenial::UnknownItem(endpoint));
            }
        }
        let key = ordered_pair(edge.a, edge.b);
        let previous = self.edges.get(&key).copied().unwrap_or(0);
        if previous == edge.weight {
            return Ok(PartitionWork::default());
        }
        let next_total = self.total_edge_weight - previous;
        self.total_edge_weight = next_total
            .checked_add(edge.weight)
            .ok_or(BisectionDenial::WeightOverflow)?;
        self.edges.insert(key, edge.weight);
        if previous == 0 {
            self.incident.get_mut(&key.0).unwrap().insert(key);
            self.incident.get_mut(&key.1).unwrap().insert(key);
        }
        Ok(PartitionWork {
            edges_visited: 1,
            ..PartitionWork::default()
        })
    }

    pub fn remove_edge(&mut self, a: PartitionItemId, b: PartitionItemId) -> PartitionWork {
        super::checked_scope::assert_offline_structural_edit(self.retained_charge.is_bound());
        let key = ordered_pair(a, b);
        if let Some(weight) = self.edges.remove(&key) {
            self.total_edge_weight -= weight;
            self.incident.get_mut(&key.0).unwrap().remove(&key);
            self.incident.get_mut(&key.1).unwrap().remove(&key);
            PartitionWork {
                edges_visited: 1,
                ..PartitionWork::default()
            }
        } else {
            PartitionWork::default()
        }
    }

    pub fn leaves(&self) -> BTreeMap<PartitionIdentity, BTreeSet<PartitionItemId>> {
        let mut result = BTreeMap::new();
        if let Some(tree) = &self.tree {
            collect_leaves(tree, &mut result);
        }
        result
    }

    pub fn cut_interfaces(
        &self,
    ) -> BTreeMap<PartitionIdentity, BTreeSet<(PartitionItemId, PartitionItemId)>> {
        let mut result = BTreeMap::new();
        if let Some(tree) = &self.tree {
            collect_interfaces(tree, &self.edges, &mut result);
        }
        result
    }

    pub fn quality(&self) -> BisectionQuality {
        let mut quality = BisectionQuality::default();
        if let Some(tree) = &self.tree {
            summarize_quality(tree, &self.edges, &mut quality);
        }
        quality
    }
}
