use std::{
    collections::{BTreeMap, BTreeSet},
    mem::size_of,
};
use worth_foundational::PartitionIdentity;

use super::{
    cut_tree::{apply_path, build_tree, planned_recut, subtree, CutTree},
    Bisection, BisectionDenial, WeightedEdge, WeightedItem,
};
use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::KernelContext,
    partition::{PartitionItemId, PartitionUpdateDenial, PartitionWork},
};

impl Bisection {
    /// Admitted runtime edge edit. Edge changes do not alter cut topology;
    /// one checkpoint precedes the bounded map/index commit.
    pub(super) fn upsert_edge_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        edge: WeightedEdge,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        if edge.a == edge.b {
            return Err(PartitionUpdateDenial::Bisection(BisectionDenial::SelfEdge(
                edge.a,
            )));
        }
        if edge.weight == 0 {
            return Err(PartitionUpdateDenial::Bisection(
                BisectionDenial::ZeroWeight,
            ));
        }
        for endpoint in [edge.a, edge.b] {
            if !self.items.contains_key(&endpoint) {
                return Err(PartitionUpdateDenial::Bisection(
                    BisectionDenial::UnknownItem(endpoint),
                ));
            }
        }
        let key = super::cut_tree::ordered_pair(edge.a, edge.b);
        let previous = self.edges.get(&key).copied().unwrap_or(0);
        if previous == edge.weight {
            return Ok(PartitionWork::default());
        }
        self.total_edge_weight
            .checked_sub(previous)
            .and_then(|sum| sum.checked_add(edge.weight))
            .ok_or(PartitionUpdateDenial::Bisection(
                BisectionDenial::WeightOverflow,
            ))?;
        let bytes = if previous == 0 {
            u64::try_from(8 * size_of::<((PartitionItemId, PartitionItemId), u64)>())
                .map_err(|_| PartitionUpdateDenial::Admission(LeaseDenial::ChargedBytesOverflow))?
        } else {
            0
        };
        let _reservation = lease
            .reserve_retained_memory(bytes)
            .map_err(PartitionUpdateDenial::Admission)?;
        let _activity = crate::backend::enter_retained_memory(bytes);
        context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
        self.upsert_edge(edge)
            .map_err(PartitionUpdateDenial::Bisection)
    }

    /// Admitted runtime edge removal; no candidate allocation is needed.
    pub(super) fn remove_edge_structural_checked(
        &mut self,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        if !self
            .edges
            .contains_key(&super::cut_tree::ordered_pair(a, b))
        {
            return Ok(PartitionWork::default());
        }
        context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
        Ok(self.remove_edge(a, b))
    }

    /// Admitted runtime edit. Every recut is built from affected-subtree edges
    /// under a scratch reservation; denial preserves the previous topology.
    pub(super) fn upsert_item_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: WeightedItem,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        self.apply_item_checked(lease, context, item.item, Some(item))
    }

    pub(super) fn remove_item_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        self.apply_item_checked(lease, context, item, None)
    }

    fn apply_item_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
        next: Option<WeightedItem>,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        if next.is_some_and(|value| value.weight == 0) {
            return Err(PartitionUpdateDenial::Bisection(
                BisectionDenial::ZeroWeight,
            ));
        }
        let previous = self.items.get(&item).copied();
        if previous == next {
            return Ok(PartitionWork::default());
        }
        if previous.is_none() && next.is_none() {
            return Ok(PartitionWork::default());
        }
        let old_weight = previous.map_or(0, |value| value.weight);
        let new_weight = next.map_or(0, |value| value.weight);
        self.tree
            .as_ref()
            .map_or(0, |tree| tree.weight)
            .checked_sub(old_weight)
            .and_then(|value| value.checked_add(new_weight))
            .ok_or(PartitionUpdateDenial::Bisection(
                BisectionDenial::WeightOverflow,
            ))?;
        if self.tree.is_none() {
            let _reservation = lease
                .reserve_retained_memory(256)
                .map_err(PartitionUpdateDenial::Admission)?;
            let _activity = crate::backend::enter_retained_memory(256);
            context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
            self.upsert_item(next.unwrap())
                .map_err(PartitionUpdateDenial::Bisection)?;
            return Ok(PartitionWork {
                members_visited: 1,
                items_rerouted: 1,
                ..PartitionWork::default()
            });
        }
        let mut work = PartitionWork::default();
        let target = if let Some(route) = self.routes.get(&item) {
            *route
        } else {
            lightest_leaf_checked(self.tree.as_ref().unwrap(), context, &mut work)?.0
        };
        let path_nodes = target.value().ilog2() - self.tree.as_ref().unwrap().path.ilog2() + 1;
        context
            .checkpoint(u64::from(path_nodes))
            .map_err(PartitionUpdateDenial::Stop)?;
        work.members_visited += u64::from(path_nodes);
        let removed_edges = if next.is_none() {
            self.incident[&item].len() as u64
        } else {
            0
        };
        context
            .checkpoint(removed_edges)
            .map_err(PartitionUpdateDenial::Stop)?;
        work.edges_visited += removed_edges;
        if next.is_none() && self.items.len() == 1 {
            context.checkpoint(2).map_err(PartitionUpdateDenial::Stop)?;
            self.remove_item(item)
                .map_err(PartitionUpdateDenial::Bisection)?;
            return Ok(PartitionWork {
                items_rerouted: 1,
                members_visited: work.members_visited + 2,
                ..work
            });
        }
        let recut = planned_recut(
            self.tree.as_ref().unwrap(),
            target.value(),
            old_weight,
            new_weight,
            self.max_leaf_weight,
            self.tolerance,
        );
        let mut _reservation = None;
        let mut _activity = None;
        let mut replacement = if let Some(path) = recut {
            let selected = subtree(self.tree.as_ref().unwrap(), path);
            let mut degree_visits = 0_usize;
            for member in &selected.members {
                context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
                work.members_visited += 1;
                degree_visits = degree_visits
                    .checked_add(self.incident[member].len())
                    .ok_or(PartitionUpdateDenial::Admission(
                        LeaseDenial::ChargedBytesOverflow,
                    ))?;
            }
            let next_members = selected.members.len() + usize::from(previous.is_none())
                - usize::from(next.is_none());
            let bytes = candidate_scratch_bytes(next_members, degree_visits, path_nodes as usize)?;
            _reservation = Some(
                lease
                    .reserve_retained_memory(bytes)
                    .map_err(PartitionUpdateDenial::Admission)?,
            );
            _activity = crate::backend::enter_retained_memory(bytes);
            context
                .checkpoint(selected.members.len() as u64)
                .map_err(PartitionUpdateDenial::Stop)?;
            work.members_visited += selected.members.len() as u64;
            let mut members = selected.members.clone();
            if next.is_none() {
                members.remove(&item);
            } else {
                members.insert(item);
            }
            let local_edges = self.local_edges_checked(&members, context, &mut work)?;
            if let Some(value) = next {
                self.items.insert(item, value);
            }
            let built = build_tree(
                path,
                members,
                &self.items,
                &local_edges,
                self.max_leaf_weight,
                &mut work,
                &mut |units| context.checkpoint(units),
            );
            if let Err(denial) = built {
                match previous {
                    Some(value) => {
                        self.items.insert(item, value);
                    }
                    None => {
                        self.items.remove(&item);
                    }
                }
                return Err(denial);
            }
            Some(built.unwrap())
        } else {
            if previous.is_none() {
                let bytes = candidate_scratch_bytes(1, 0, path_nodes as usize)?;
                _reservation = Some(
                    lease
                        .reserve_retained_memory(bytes)
                        .map_err(PartitionUpdateDenial::Admission)?,
                );
                _activity = crate::backend::enter_retained_memory(bytes);
            }
            None
        };
        // Direct route publication visits fewer than 2m cut nodes and writes
        // at most m leaf routes, then updates ancestors and removed edges.
        // Admit the entire non-interruptible commit before retained mutation.
        let route_members = replacement
            .as_ref()
            .map_or(0, |tree| tree.members.len() as u64);
        let commit_bounds = route_members
            .checked_mul(3)
            .and_then(|routes| u64::from(path_nodes).checked_add(routes))
            .and_then(|members| removed_edges.checked_mul(2).map(|edges| (members, edges)))
            .and_then(|(members, edges)| {
                members
                    .checked_add(edges)
                    .and_then(|value| value.checked_add(2))
                    .map(|units| (members, edges, units))
            });
        let Some((commit_members, commit_edges, commit_units)) = commit_bounds else {
            if recut.is_some() && next.is_some() {
                match previous {
                    Some(value) => {
                        self.items.insert(item, value);
                    }
                    None => {
                        self.items.remove(&item);
                    }
                }
            }
            return Err(PartitionUpdateDenial::Stop(
                crate::backend::KernelStop::WorkCounterOverflow,
            ));
        };
        if let Err(stop) = context.checkpoint(commit_units) {
            if recut.is_some() && next.is_some() {
                match previous {
                    Some(value) => {
                        self.items.insert(item, value);
                    }
                    None => {
                        self.items.remove(&item);
                    }
                }
            }
            return Err(PartitionUpdateDenial::Stop(stop));
        }
        work.members_visited += commit_members;
        work.edges_visited += commit_edges;
        // Candidate construction and every stop point precede this commit.
        apply_path(
            self.tree.as_mut().unwrap(),
            target.value(),
            item,
            old_weight,
            new_weight,
            recut,
            &mut replacement,
            &mut self.routes,
            &mut work,
        );
        match next {
            Some(value) => {
                self.items.insert(item, value);
                self.incident.entry(item).or_default();
            }
            None => {
                self.items.remove(&item);
                self.routes.remove(&item);
                work.items_rerouted += 1;
                for pair in self.incident.remove(&item).unwrap() {
                    self.total_edge_weight -= self.edges.remove(&pair).unwrap();
                    let other = if pair.0 == item { pair.1 } else { pair.0 };
                    self.incident.get_mut(&other).unwrap().remove(&pair);
                }
            }
        }
        Ok(work)
    }

    fn local_edges_checked(
        &self,
        members: &BTreeSet<PartitionItemId>,
        context: &mut KernelContext<'_, '_>,
        work: &mut PartitionWork,
    ) -> Result<BTreeMap<(PartitionItemId, PartitionItemId), u64>, PartitionUpdateDenial> {
        let mut edges = BTreeMap::new();
        for member in members {
            if let Some(pairs) = self.incident.get(member) {
                for pair in pairs {
                    context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
                    work.edges_visited += 1;
                    if pair.0 == *member && members.contains(&pair.1) {
                        edges.insert(*pair, self.edges[pair]);
                    }
                }
            }
        }
        Ok(edges)
    }
}

fn lightest_leaf_checked(
    tree: &CutTree,
    context: &mut KernelContext<'_, '_>,
    work: &mut PartitionWork,
) -> Result<(PartitionIdentity, u64), PartitionUpdateDenial> {
    context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
    work.members_visited += 1;
    let Some((left, right)) = &tree.children else {
        return Ok((PartitionIdentity::new(tree.path), tree.weight));
    };
    let left = lightest_leaf_checked(left, context, work)?;
    let right = lightest_leaf_checked(right, context, work)?;
    Ok(if left.1 <= right.1 { left } else { right })
}

fn candidate_scratch_bytes(
    members: usize,
    degree_visits: usize,
    path_nodes: usize,
) -> Result<u64, PartitionUpdateDenial> {
    // At most members levels, with a member set and one candidate left slice
    // per level. The factor reserves BTree node slack and route entries.
    let per_member = 16 * size_of::<(PartitionItemId, worth_foundational::PartitionIdentity)>();
    let per_edge = 8 * size_of::<((PartitionItemId, PartitionItemId), u64)>();
    members
        .checked_mul(members)
        .and_then(|count| count.checked_mul(per_member))
        .and_then(|bytes| bytes.checked_add(degree_visits.checked_mul(per_edge)?))
        .and_then(|bytes| bytes.checked_add(path_nodes.checked_mul(per_member)?))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(PartitionUpdateDenial::Admission(
            LeaseDenial::ChargedBytesOverflow,
        ))
}
