use std::{
    collections::{BTreeMap, BTreeSet},
    mem::size_of,
};

use worth_foundational::PartitionIdentity;

use super::ComponentPartitioner;
use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::KernelContext,
    partition::{PartitionItemId, PartitionUpdateDenial, PartitionWork},
};

// Candidate parent links, new routes, and rebuilt membership sets coexist
// with retained state. The per-member factor provides sixteen pair slots for
// parent/route maps plus eight for component keys and membership. This covers
// even a separate small BTree allocation per split-off member, including node
// slack and allocator headers. The per-degree factor covers eight item slots
// for each incident reference. Checked arithmetic denies oversized candidates
// before allocation; these are reservation ceilings, not measured heap usage.
pub(super) const MEMBER_SCRATCH_BYTES: usize = 16 * size_of::<(PartitionItemId, PartitionItemId)>()
    + 8 * size_of::<(PartitionIdentity, PartitionItemId)>();
pub(super) const DEGREE_SCRATCH_BYTES: usize = 8 * size_of::<PartitionItemId>();

impl ComponentPartitioner {
    /// Admitted runtime item edit. The fixed reservation covers one new key in
    /// each retained map and its singleton membership/adjacency BTree nodes.
    pub(super) fn upsert_item_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        if self.routes.contains_key(&item) {
            return Ok(PartitionWork::default());
        }
        let bytes = u64::try_from(MEMBER_SCRATCH_BYTES)
            .map_err(|_| PartitionUpdateDenial::Admission(LeaseDenial::ChargedBytesOverflow))?;
        let _reservation = lease
            .reserve_retained_memory(bytes)
            .map_err(PartitionUpdateDenial::Admission)?;
        let _activity = crate::backend::enter_retained_memory(bytes);
        context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
        Ok(self.upsert_item(item))
    }

    /// Checked incremental merge. Every losing-island member is admitted
    /// before either adjacency or route changes.
    pub(super) fn add_edge_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        self.check_edge(a, b)
            .map_err(PartitionUpdateDenial::Components)?;
        if self.adjacent[&a].contains(&b) {
            return Ok(PartitionWork::default());
        }
        let a_identity = self.routes[&a];
        let b_identity = self.routes[&b];
        let losing = if a_identity < b_identity {
            b_identity
        } else {
            a_identity
        };
        let moved = if a_identity == b_identity {
            0
        } else {
            self.members[&losing].len()
        };
        let bytes = moved
            .checked_mul(MEMBER_SCRATCH_BYTES)
            .and_then(|value| value.checked_add(2 * DEGREE_SCRATCH_BYTES))
            .and_then(|value| u64::try_from(value).ok())
            .ok_or(PartitionUpdateDenial::Admission(
                LeaseDenial::ChargedBytesOverflow,
            ))?;
        let _reservation = lease
            .reserve_retained_memory(bytes)
            .map_err(PartitionUpdateDenial::Admission)?;
        let _activity = crate::backend::enter_retained_memory(bytes);
        context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
        if moved != 0 {
            for _ in &self.members[&losing] {
                context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
            }
        }
        // The final merge cannot stop between route rewrites and membership
        // extension. It performs at most two member traversals plus two
        // adjacency insertions; admit that entire atomic commit in advance.
        let moved_units = u64::try_from(moved).map_err(|_| {
            PartitionUpdateDenial::Stop(crate::backend::KernelStop::WorkCounterOverflow)
        })?;
        let commit_units = moved_units
            .checked_mul(2)
            .and_then(|value| value.checked_add(2))
            .ok_or(PartitionUpdateDenial::Stop(
                crate::backend::KernelStop::WorkCounterOverflow,
            ))?;
        context
            .checkpoint(commit_units)
            .map_err(PartitionUpdateDenial::Stop)?;
        let mut work = self
            .add_edge(a, b)
            .map_err(PartitionUpdateDenial::Components)?;
        work.members_visited = work
            .members_visited
            .checked_add(moved_units * 2)
            .expect("commit units checked before mutation");
        work.edges_visited += 2;
        Ok(work)
    }

    /// Checked removal of a coupling edge. The affected island is rebuilt as a
    /// candidate; denial leaves the retained graph and routes unchanged.
    pub(super) fn remove_edge_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        self.check_edge(a, b)
            .map_err(PartitionUpdateDenial::Components)?;
        if !self.adjacent[&a].contains(&b) {
            return Ok(PartitionWork::default());
        }
        let identity = self.routes[&a];
        let work = self.rebuild_checked(lease, context, identity, None, Some((a, b)))?;
        self.adjacent.get_mut(&a).unwrap().remove(&b);
        self.adjacent.get_mut(&b).unwrap().remove(&a);
        self.edge_count -= 1;
        Ok(work)
    }

    /// Checked removal of one item and its incident edges. Least-member
    /// renaming and splits affect only the old island.
    pub(super) fn remove_item_structural_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        context.checkpoint(0).map_err(PartitionUpdateDenial::Stop)?;
        let Some(&identity) = self.routes.get(&item) else {
            return Ok(PartitionWork::default());
        };
        let mut work = self.rebuild_checked(lease, context, identity, Some(item), None)?;
        let neighbors = self.adjacent.remove(&item).unwrap();
        self.edge_count -= neighbors.len();
        for neighbor in neighbors {
            self.adjacent.get_mut(&neighbor).unwrap().remove(&item);
        }
        self.routes.remove(&item);
        work.items_rerouted += 1;
        Ok(work)
    }

    fn rebuild_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        identity: PartitionIdentity,
        excluded_item: Option<PartitionItemId>,
        excluded_edge: Option<(PartitionItemId, PartitionItemId)>,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let old = &self.members[&identity];
        let mut work = PartitionWork::default();
        let mut degrees = 0_usize;
        for item in old {
            context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
            work.members_visited += 1;
            degrees = degrees.checked_add(self.adjacent[item].len()).ok_or(
                PartitionUpdateDenial::Admission(
                    crate::authority::LeaseDenial::ChargedBytesOverflow,
                ),
            )?;
        }
        // Parent links, rebuilt membership, and the transient candidate are
        // conservatively reserved before the first candidate allocation.
        let bytes = old
            .len()
            .checked_mul(MEMBER_SCRATCH_BYTES)
            .and_then(|value| value.checked_add(degrees.checked_mul(DEGREE_SCRATCH_BYTES)?))
            .and_then(|value| u64::try_from(value).ok())
            .ok_or(PartitionUpdateDenial::Admission(
                crate::authority::LeaseDenial::ChargedBytesOverflow,
            ))?;
        let _reservation = lease
            .reserve_retained_memory(bytes)
            .map_err(PartitionUpdateDenial::Admission)?;
        let _activity = crate::backend::enter_retained_memory(bytes);

        let mut parent: BTreeMap<_, _> = BTreeMap::new();
        for item in old {
            if Some(*item) == excluded_item {
                continue;
            }
            context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
            work.members_visited += 1;
            parent.insert(*item, *item);
        }
        for item in old {
            if Some(*item) == excluded_item {
                continue;
            }
            for neighbor in &self.adjacent[item] {
                context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
                work.edges_visited += 1;
                if Some(*neighbor) == excluded_item
                    || !old.contains(neighbor)
                    || item >= neighbor
                    || excluded_edge.is_some_and(|(a, b)| {
                        (*item == a && *neighbor == b) || (*item == b && *neighbor == a)
                    })
                {
                    continue;
                }
                checked_union(&mut parent, *item, *neighbor, context, &mut work)?;
            }
        }
        let mut rebuilt: BTreeMap<PartitionIdentity, BTreeSet<PartitionItemId>> = BTreeMap::new();
        let mut new_routes = BTreeMap::new();
        for item in old {
            if Some(*item) == excluded_item {
                continue;
            }
            context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
            work.members_visited += 1;
            let next =
                PartitionIdentity::new(checked_root(&mut parent, *item, context, &mut work)?.0);
            if self.routes[item] != next {
                work.items_rerouted += 1;
            }
            rebuilt.entry(next).or_default().insert(*item);
            new_routes.insert(*item, next);
        }
        // Publication traverses the old island to replace routes and moves the
        // rebuilt membership sets. Edge/item cleanup is a second bounded
        // atomic phase, admitted here before any retained state changes.
        let members = u64::try_from(old.len()).map_err(|_| {
            PartitionUpdateDenial::Stop(crate::backend::KernelStop::WorkCounterOverflow)
        })?;
        let cleanup_edges = if let Some(item) = excluded_item {
            u64::try_from(self.adjacent[&item].len()).map_err(|_| {
                PartitionUpdateDenial::Stop(crate::backend::KernelStop::WorkCounterOverflow)
            })?
        } else {
            2
        };
        let commit_units = members
            .checked_mul(2)
            .and_then(|value| value.checked_add(cleanup_edges))
            .ok_or(PartitionUpdateDenial::Stop(
                crate::backend::KernelStop::WorkCounterOverflow,
            ))?;
        context
            .checkpoint(commit_units)
            .map_err(PartitionUpdateDenial::Stop)?;
        work.members_visited += members * 2;
        work.edges_visited += cleanup_edges;
        // No cancellation, admission, or fallible allocation check follows.
        self.members.remove(&identity);
        self.members.extend(rebuilt);
        self.routes.extend(new_routes);
        Ok(work)
    }
}

fn checked_root(
    parent: &mut BTreeMap<PartitionItemId, PartitionItemId>,
    item: PartitionItemId,
    context: &mut KernelContext<'_, '_>,
    work: &mut PartitionWork,
) -> Result<PartitionItemId, PartitionUpdateDenial> {
    let mut root = item;
    while parent[&root] != root {
        context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
        work.members_visited += 1;
        root = parent[&root];
    }
    let mut cursor = item;
    while parent[&cursor] != root {
        context.checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
        work.members_visited += 1;
        let next = parent[&cursor];
        parent.insert(cursor, root);
        cursor = next;
    }
    Ok(root)
}

fn checked_union(
    parent: &mut BTreeMap<PartitionItemId, PartitionItemId>,
    a: PartitionItemId,
    b: PartitionItemId,
    context: &mut KernelContext<'_, '_>,
    work: &mut PartitionWork,
) -> Result<(), PartitionUpdateDenial> {
    let a = checked_root(parent, a, context, work)?;
    let b = checked_root(parent, b, context, work)?;
    if a != b {
        let (least, other) = if a < b { (a, b) } else { (b, a) };
        parent.insert(other, least);
    }
    Ok(())
}
