//! Persistent canonical order; ranks are views, never stored suffix positions.
use super::*;
use worth_execution::ExecutionOrderedMap;
pub(in crate::domain_computation::primary_graph) struct CallShape {
    pub(super) order: ExecutionOrderedMap<ComputationRead, ()>,
    pub(super) prefix: usize,
    pub(super) _memory: QueryMemoryReservation,
}
impl CallShape {
    #[cfg(test)]
    pub(super) fn bytes(&self) -> u64 {
        self._memory
            .bytes()
            .checked_add(self.order.retained_bytes())
            .expect("actual test backing sizes fit")
    }
    pub(in crate::domain_computation::primary_graph) fn calls(
        &self,
    ) -> impl Iterator<Item = ComputationRead> + '_ {
        self.order.keys().copied()
    }
    pub(in crate::domain_computation::primary_graph) fn ordinal(
        &self,
        read: ComputationRead,
    ) -> Option<usize> {
        self.order.rank_of(&read)
    }
    pub(in crate::domain_computation::primary_graph) fn call_at(
        &self,
        ordinal: usize,
    ) -> Option<ComputationRead> {
        self.order.entry_at(ordinal).map(|(read, _)| *read)
    }
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.order.len()
    }
    pub(in crate::domain_computation::primary_graph) fn lower_bound(
        &self,
        read: ComputationRead,
    ) -> usize {
        self.order
            .first_at_or_after(&read)
            .map_or(self.len(), |(next, _)| {
                self.order.rank_of(next).expect("present next call")
            })
    }
    /// Prepare identity correspondence before fresh gathers produce charges.
    pub(in crate::domain_computation::primary_graph) fn prepare_transition(
        self: &Arc<Self>,
        inserted: &BTreeSet<ComputationRead>,
        retired: &BTreeSet<ComputationRead>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<CallShapeTransition>, CallSequenceDenial> {
        let policy = execution.allocation_policy()?;
        let mut order = self.order.clone();
        let (mut added, mut removed) = (
            ExecutionOrderedMap::default(),
            ExecutionOrderedMap::default(),
        );
        let mut prefix = self.prefix;
        for read in retired {
            if inserted.contains(read) || !order.remove_in(read, policy)? {
                return Err(CallSequenceDenial::InvalidEdit);
            }
            removed.insert_cloned_in(read, &(), policy)?;
            prefix = prefix
                .checked_sub(usize::from(!matches!(read, ComputationRead::Partition(_))))
                .ok_or(Resource::CapacityOverflow)?;
        }
        for read in inserted {
            if order.contains_key(read) {
                return Err(CallSequenceDenial::InvalidEdit);
            }
            order.insert_cloned_in(read, &(), policy)?;
            added.insert_cloned_in(read, &(), policy)?;
            prefix = prefix
                .checked_add(usize::from(!matches!(read, ComputationRead::Partition(_))))
                .ok_or(Resource::CapacityOverflow)?;
        }
        let target = if added.is_empty() && removed.is_empty() {
            Arc::clone(self)
        } else {
            Self::issue(order, prefix, execution)?
        };
        let memory = execution
            .reserve((size_of::<CallShapeTransition>() + 2 * size_of::<usize>()) as u64)?;
        Ok(Arc::new(CallShapeTransition {
            prior: Arc::clone(self),
            target,
            inserted: added,
            retired: removed,
            _memory: memory,
        }))
    }
    pub(super) fn issue(
        ordinals: ExecutionOrderedMap<ComputationRead, ()>,
        prefix: usize,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, Resource> {
        let memory = execution.reserve((size_of::<Self>() + 2 * size_of::<usize>()) as u64)?;
        Ok(Arc::new(Self {
            order: ordinals,
            prefix,
            _memory: memory,
        }))
    }
}
/// Issued only by the sequence's checked edits. It describes correspondence;
/// the native prior and successful completed run must independently bind it.
pub(in crate::domain_computation::primary_graph) struct CallShapeTransition {
    pub(super) prior: Arc<CallShape>,
    pub(super) target: Arc<CallShape>,
    pub(super) inserted: ExecutionOrderedMap<ComputationRead, ()>,
    pub(super) retired: ExecutionOrderedMap<ComputationRead, ()>,
    pub(super) _memory: QueryMemoryReservation,
}
impl CallShapeTransition {
    pub(in crate::domain_computation::primary_graph) fn matches_prior(
        &self,
        shape: &Arc<CallShape>,
    ) -> bool {
        Arc::ptr_eq(&self.prior, shape)
    }
    pub(in crate::domain_computation::primary_graph) fn target(&self) -> &Arc<CallShape> {
        &self.target
    }
    pub(in crate::domain_computation::primary_graph) fn inserted(
        &self,
    ) -> impl Iterator<Item = ComputationRead> + '_ {
        self.inserted.keys().copied()
    }
    pub(in crate::domain_computation::primary_graph) fn retired(
        &self,
    ) -> impl Iterator<Item = ComputationRead> + '_ {
        self.retired.keys().copied()
    }
    pub(in crate::domain_computation::primary_graph) fn closes_fresh(
        &self,
        fresh: &BTreeSet<ComputationRead>,
    ) -> bool {
        self.inserted().all(|read| fresh.contains(&read))
            && self.retired().all(|read| !fresh.contains(&read))
    }
    pub(in crate::domain_computation::primary_graph) fn maps(
        &self,
        prior: Range<usize>,
        target: Range<usize>,
    ) -> bool {
        if prior.start >= prior.end
            || target.start >= target.end
            || prior.end > self.prior.len()
            || target.end > self.target.len()
            || prior.len() != target.len()
            || self.prior.call_at(prior.start) != self.target.call_at(target.start)
            || self.prior.call_at(prior.end - 1) != self.target.call_at(target.end - 1)
        {
            return false;
        }
        let first = self.prior.call_at(prior.start).unwrap();
        let last = self.prior.call_at(prior.end - 1).unwrap();
        for edits in [&self.inserted, &self.retired] {
            if edits
                .first_at_or_after(&first)
                .is_some_and(|(read, _)| *read <= last)
            {
                return false;
            }
        }
        true
    }
}
