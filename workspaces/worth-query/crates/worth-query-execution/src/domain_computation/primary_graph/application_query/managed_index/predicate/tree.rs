//! One funded mutable record set, addressed by genuine immutable registration slots.
mod balance;
mod removal;
#[cfg(test)]
mod tests;
use super::WorthQueryManagedIndexPredicateSummary as Summary;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexStop as Stop,
};
use std::{cmp::Ordering, collections::BTreeMap};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

/// Issued by the registration owner; equal probes retain distinct exact slots.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WatchSlot(
    pub(in crate::domain_computation::primary_graph::application_query) u64,
);

pub(super) struct Record<P, R, S> {
    pub(super) probe: P,
    pub(super) receiver: R,
    own: S,
    aggregate: S,
    parent: Option<WatchSlot>,
    children: [Option<WatchSlot>; 2],
    extrema: [WatchSlot; 2],
    height: u8,
    // Probe/receiver fields and actual tree backing close before this grant.
    _memory: Option<ExecutionMemoryReservation>,
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct PredicateTree<
    P,
    R,
    S,
> {
    root: Option<WatchSlot>,
    records: BTreeMap<WatchSlot, Record<P, R, S>>,
}
impl<P, R, S> Default for PredicateTree<P, R, S> {
    fn default() -> Self {
        Self {
            root: None,
            records: BTreeMap::new(),
        }
    }
}
impl<P, R, S> Drop for PredicateTree<P, R, S> {
    fn drop(&mut self) {
        while let Some((_, record)) = self.records.pop_first() {
            if self.records.is_empty() {
                drop(std::mem::take(&mut self.records));
            }
            drop(record);
        }
    }
}

/// Backing (including an empty last map) closes before its last original grant.
pub(super) struct Retired<P, R, S> {
    _empty_backing: Option<BTreeMap<WatchSlot, Record<P, R, S>>>,
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    pub(super) record: Record<P, R, S>,
}

impl<P, R, S: Summary> PredicateTree<P, R, S> {
    fn node(&self, id: WatchSlot) -> &Record<P, R, S> {
        // Private links are installed only with their actual owned record and
        // unlinked before removal; callers cannot forge a reachable coordinate.
        self.records.get(&id).expect("reachable predicate record")
    }
    fn node_mut(&mut self, id: WatchSlot) -> &mut Record<P, R, S> {
        self.records
            .get_mut(&id)
            .expect("reachable predicate record")
    }
    fn height(&self, id: Option<WatchSlot>) -> u8 {
        id.map_or(0, |id| self.node(id).height)
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }
    pub(super) fn slots(&self) -> impl Iterator<Item = WatchSlot> + '_ {
        self.records.keys().copied()
    }

    fn topology_work(&self) -> Result<u64, Stop> {
        let count = self
            .records
            .len()
            .checked_add(1)
            .ok_or(Stop::CapacityOverflow)?;
        let levels = u64::from(usize::BITS - count.leading_zeros());
        // Each edit performs a fixed number of map accesses per AVL ancestor;
        // each mutable BTree lookup has at most 11 comparisons per level.
        levels
            .checked_mul(11 * 64)
            .and_then(|work| work.checked_mul(u64::from(self.height(self.root)) + 2))
            .ok_or(Stop::CapacityOverflow)
    }

    pub(super) fn insert<E: From<Stop>>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        slot: WatchSlot,
        probe: P,
        receiver: R,
        own: S,
        control: &Control<'_>,
        mut compare: impl FnMut(&P, &P) -> Result<Ordering, E>,
    ) -> Result<(), E> {
        control.checkpoint(self.topology_work()?)?;
        if self.records.contains_key(&slot) {
            return Err(Stop::ForeignNode.into());
        }
        let location = self.locate(slot, &probe, &mut compare)?;
        // Original grant quotes actual fixed-layout maximum BTree split backing.
        // Probe/receiver heaps have their own original grants, carried in P/R.
        let bytes = size_of::<(WatchSlot, Record<P, R, S>)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(14 * size_of::<usize>()))
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Stop::CapacityOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Stop::Request)?;
        control.poll()?;
        self.attach(
            slot,
            Record {
                probe,
                receiver,
                own,
                aggregate: own,
                parent: None,
                children: [None; 2],
                extrema: [slot; 2],
                height: 1,
                _memory: memory,
            },
            location,
        );
        Ok(())
    }

    fn locate<E>(
        &self,
        slot: WatchSlot,
        probe: &P,
        compare: &mut impl FnMut(&P, &P) -> Result<Ordering, E>,
    ) -> Result<(Option<WatchSlot>, usize), E> {
        let mut parent = None;
        let mut side = 0;
        let mut next = self.root;
        while let Some(id) = next {
            let node = self.node(id);
            let order = compare(probe, &node.probe)?.then_with(|| slot.cmp(&id));
            parent = Some(id);
            side = usize::from(order == Ordering::Greater);
            next = node.children[side];
        }
        Ok((parent, side))
    }

    fn attach(
        &mut self,
        slot: WatchSlot,
        mut record: Record<P, R, S>,
        location: (Option<WatchSlot>, usize),
    ) {
        let (parent, side) = location;
        record.parent = parent;
        self.records.insert(slot, record);
        if let Some(parent) = parent {
            self.node_mut(parent).children[side] = Some(slot);
        } else {
            self.root = Some(slot);
        }
        self.rebalance(parent);
    }

    /// Visit only reachable records. A rejected enclosure prunes both children.
    /// The receiver callback validates this exact node's predicate separately.
    pub(super) fn visit<E: From<Stop>>(
        &self,
        control: &Control<'_>,
        region: &mut impl FnMut(S, [&P; 2]) -> Result<bool, E>,
        receive: &mut impl FnMut(WatchSlot, &P, &R) -> Result<(), E>,
    ) -> Result<(), E> {
        self.walk(
            self.root,
            &mut |work| control.checkpoint(work).map_err(E::from),
            region,
            receive,
        )
    }

    fn walk<E>(
        &self,
        id: Option<WatchSlot>,
        checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
        region: &mut impl FnMut(S, [&P; 2]) -> Result<bool, E>,
        receive: &mut impl FnMut(WatchSlot, &P, &R) -> Result<(), E>,
    ) -> Result<(), E> {
        let Some(id) = id else {
            return Ok(());
        };
        let levels = u64::from(usize::BITS - self.records.len().leading_zeros());
        checkpoint(44 * levels)?;
        let node = self.node(id);
        if !region(
            node.aggregate,
            [
                &self.node(node.extrema[0]).probe,
                &self.node(node.extrema[1]).probe,
            ],
        )? {
            return Ok(());
        }
        receive(id, &node.probe, &node.receiver)?;
        self.walk(node.children[0], checkpoint, region, receive)?;
        self.walk(node.children[1], checkpoint, region, receive)
    }
}
