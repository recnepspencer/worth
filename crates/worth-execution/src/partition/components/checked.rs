use std::mem::size_of;

use worth_foundational::PartitionIdentity;

use super::{request::ComponentFunding, ComponentPartitioner};
use crate::{
    authority::LeaseDenial,
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
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        funding.checkpoint(context, 0)?;
        if self.routes.contains_key(&item) {
            return Ok(PartitionWork::default());
        }
        let bytes = u64::try_from(MEMBER_SCRATCH_BYTES)
            .map_err(|_| PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow()))?;
        let _reservation = funding.scratch(bytes)?;
        let _activity = funding.activity(bytes);
        funding.checkpoint(context, 1)?;
        self.check_group_capacity(1).ok_or_else(|| {
            PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
        })?;
        Ok(self.upsert_item(item))
    }

    /// Checked incremental merge. Every losing-island member is admitted
    /// before either adjacency or route changes.
    pub(super) fn add_edge_structural_checked(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        funding.checkpoint(context, 0)?;
        self.check_edge(a, b)
            .map_err(PartitionUpdateDenial::Components)?;
        if self.adjacent[&a].contains(&b) {
            return Ok(PartitionWork::default());
        }
        let a_identity = self.route(a).unwrap();
        let b_identity = self.route(b).unwrap();
        let (_, losing) = self.merge_sides(a_identity, b_identity);
        let moved = if a_identity == b_identity {
            0
        } else {
            self.members[&losing].len()
        };
        let bytes = moved
            .checked_mul(MEMBER_SCRATCH_BYTES)
            .and_then(|value| value.checked_add(2 * DEGREE_SCRATCH_BYTES))
            .and_then(|value| u64::try_from(value).ok())
            .ok_or_else(|| {
                PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
            })?;
        let _reservation = funding.scratch(bytes)?;
        let _activity = funding.activity(bytes);
        funding.checkpoint(context, 1)?;
        if moved != 0 {
            for _ in &self.members[&losing] {
                funding.checkpoint(context, 1)?;
            }
        }
        // The final merge cannot stop between route rewrites and membership
        // extension. It performs at most two member traversals plus two
        // adjacency insertions; admit that entire atomic commit in advance.
        let moved_units = u64::try_from(moved).map_err(|_| {
            PartitionUpdateDenial::Stop(crate::backend::KernelStop::work_counter_overflow())
        })?;
        let commit_units = moved_units
            .checked_mul(2)
            .and_then(|value| value.checked_add(2))
            .ok_or_else(|| {
                PartitionUpdateDenial::Stop(crate::backend::KernelStop::work_counter_overflow())
            })?;
        funding.checkpoint(context, commit_units)?;
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

    /// Checked removal stages forest-cut discovery and replacement edges;
    /// denial leaves the retained graph and routing witness unchanged.
    pub(super) fn remove_edge_structural_checked(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        funding.checkpoint(context, 0)?;
        self.check_edge(a, b)
            .map_err(PartitionUpdateDenial::Components)?;
        if !self.adjacent[&a].contains(&b) {
            return Ok(PartitionWork::default());
        }
        let identity = self.route(a).unwrap();
        self.remove_structural_checked(funding, context, identity, None, Some((a, b)))
    }

    /// Discover only the forest pieces affected by the removed item's edges.
    pub(super) fn remove_item_structural_checked(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        funding.checkpoint(context, 0)?;
        let Some(identity) = self.route(item) else {
            return Ok(PartitionWork::default());
        };
        self.remove_structural_checked(funding, context, identity, Some(item), None)
    }

    fn remove_structural_checked(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        identity: PartitionIdentity,
        item: Option<PartitionItemId>,
        edge: Option<(PartitionItemId, PartitionItemId)>,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let seeds = item.map_or(2, |item| self.forest[&item].len());
        let mut discovery = ChargedDiscovery::new(funding, context, seeds)?;
        let mut plan = self.plan_removal(item, edge, &mut discovery)?;
        self.check_group_capacity(plan.groups.len())
            .ok_or_else(overflow)?;
        let changed = plan
            .groups
            .iter()
            .try_fold(0u64, |n, g| n.checked_add(g.len() as u64))
            .ok_or_else(overflow)?;
        let cleanup = item.map_or(2, |item| self.adjacent[&item].len() as u64);
        let effects = changed
            .checked_mul(2)
            .and_then(|n| n.checked_add(cleanup))
            .and_then(|n| n.checked_add((plan.forest_edges.len() as u64).checked_mul(2)?))
            .and_then(|n| n.checked_add(1))
            .ok_or_else(overflow)?;
        discovery.funding.checkpoint(discovery.context, effects)?;
        plan.work.members_visited = plan
            .work
            .members_visited
            .checked_add(changed)
            .ok_or_else(overflow)?;
        plan.work.edges_visited = plan
            .work
            .edges_visited
            .checked_add(cleanup)
            .ok_or_else(overflow)?;
        Ok(self.apply_removal(identity, item, edge, plan))
    }
}

fn overflow() -> PartitionUpdateDenial {
    PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
}

struct ChargedDiscovery<'stage, 'lease, 'lease_owner, 'meter, 'meter_owner> {
    funding: &'stage ComponentFunding<'lease, 'lease_owner>,
    context: &'stage mut KernelContext<'meter, 'meter_owner>,
    memory: Option<crate::ExecutionMemoryReservation>,
    activity: Option<crate::backend::MemoryActivityGuard>,
    capacity: usize,
    visits: usize,
}
impl<'stage, 'lease, 'lease_owner, 'meter, 'meter_owner>
    ChargedDiscovery<'stage, 'lease, 'lease_owner, 'meter, 'meter_owner>
{
    fn new(
        funding: &'stage ComponentFunding<'lease, 'lease_owner>,
        context: &'stage mut KernelContext<'meter, 'meter_owner>,
        seeds: usize,
    ) -> Result<Self, PartitionUpdateDenial> {
        let capacity = seeds
            .max(1)
            .checked_next_power_of_two()
            .ok_or_else(overflow)?;
        let bytes = scratch_bytes(capacity)?;
        Ok(Self {
            funding,
            context,
            memory: funding.scratch(bytes)?,
            activity: funding.activity(bytes),
            capacity,
            visits: 0,
        })
    }
}
impl super::discovery::Discovery for ChargedDiscovery<'_, '_, '_, '_, '_> {
    type Error = PartitionUpdateDenial;
    fn member(&mut self) -> Result<(), Self::Error> {
        let next = self.visits.checked_add(1).ok_or_else(overflow)?;
        if next > self.capacity {
            let capacity = self.capacity.checked_mul(2).ok_or_else(overflow)?;
            let bytes = scratch_bytes(capacity)?;
            if let Some(memory) = &mut self.memory {
                memory.resize(bytes).map_err(|cause| {
                    PartitionUpdateDenial::Admission(LeaseDenial::MemoryExhausted(cause))
                })?;
            }
            self.activity.take();
            self.activity = self.funding.activity(bytes);
            self.capacity = capacity;
        }
        self.funding.checkpoint(self.context, 1)?;
        self.visits = next;
        Ok(())
    }
    fn edge(&mut self) -> Result<(), Self::Error> {
        self.funding.checkpoint(self.context, 1)
    }
}
fn scratch_bytes(count: usize) -> Result<u64, PartitionUpdateDenial> {
    count
        .checked_mul(MEMBER_SCRATCH_BYTES)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(overflow)
}
