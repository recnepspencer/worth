use super::super::{recombine::NodeEngine, Link, Node, ReductionRunStop};
use super::RunSummary;
use crate::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionResourceLease, ExecutionRetainedResult,
    LeaseDenial, RetainedRunMetrics,
};
use std::sync::Arc;
use worth_foundational::PartitionIdentity;

pub(super) struct Slot<R> {
    pub(super) result: Option<ExecutionRetainedResult<R>>,
    pub(super) summary: RunSummary,
}
impl<R> Clone for Slot<R> {
    fn clone(&self) -> Self {
        Self {
            result: self.result.clone(),
            summary: self.summary,
        }
    }
}
impl<R> Slot<R> {
    pub(super) fn new(value: ExecutionRetainedResult<R>) -> Self {
        Self {
            summary: RunSummary::slot(&value),
            result: Some(value),
        }
    }
}
impl<R> ChargedBytes for Slot<R> {
    fn additional_charged_bytes(&self) -> u64 {
        self.result
            .as_ref()
            .map_or(0, ChargedBytes::additional_charged_bytes)
    }
}
pub(super) struct RunEngine<'a, 'lease> {
    pub(super) lease: Option<&'a ExecutionResourceLease<'lease>>,
    pub(super) metrics: &'a mut RetainedRunMetrics,
}
impl<R> NodeEngine<Slot<R>, LeaseDenial> for RunEngine<'_, '_> {
    fn inspect(&mut self) {
        self.metrics.edit_visits += 1;
    }
    fn validate(&self, _: &Slot<R>) -> Result<(), ReductionRunStop<LeaseDenial>> {
        Ok(())
    }
    fn same(&self, a: &Slot<R>, b: &Slot<R>) -> Result<bool, ReductionRunStop<LeaseDenial>> {
        // Exact native backing is only an immutable-storage cutoff. No R
        // comparison, digest, source authority or application freshness grant.
        Ok(match (&a.result, &b.result) {
            (Some(a), Some(b)) => a.same_backing(b),
            (None, None) => a.summary == b.summary,
            _ => false,
        })
    }
    fn build(
        &mut self,
        id: PartitionIdentity,
        value: Slot<R>,
        left: Link<Slot<R>>,
        right: Link<Slot<R>>,
    ) -> Result<Arc<Node<Slot<R>>>, ReductionRunStop<LeaseDenial>> {
        let summary = left
            .as_ref()
            .map_or_else(RunSummary::empty, |n| n.aggregate.summary)
            .append(value.summary)
            .and_then(|v| {
                v.append(
                    right
                        .as_ref()
                        .map_or_else(RunSummary::empty, |n| n.aggregate.summary),
                )
            })
            .ok_or_else(|| ReductionRunStop::Hook(LeaseDenial::charged_bytes_overflow()))?;
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Node<Slot<R>>>())
            .map_err(|_| ReductionRunStop::Hook(LeaseDenial::charged_bytes_overflow()))?
            .0
            .pad_to_align()
            .size() as u64;
        // Grants cover only this node; result/child cells already own theirs.
        // No root keeps an obsolete parent or its unrelated allocations alive.
        let memory = ExecutionMemoryReservation::reserve_in_scope(self.lease, bytes)
            .map_err(ReductionRunStop::Hook)?;
        let retained_bytes = bytes
            .checked_add(value.additional_charged_bytes())
            .and_then(|v| v.checked_add(left.as_ref().map_or(0, |n| n.retained_bytes)))
            .and_then(|v| v.checked_add(right.as_ref().map_or(0, |n| n.retained_bytes)))
            .ok_or_else(|| ReductionRunStop::Hook(LeaseDenial::charged_bytes_overflow()))?;
        self.metrics.copied_nodes += 1;
        Ok(Arc::new(Node {
            identity: id,
            value,
            aggregate: Slot {
                result: None,
                summary,
            },
            left,
            right,
            retained_bytes,
            full_subtree_span: 0,
            _memory: Some(memory),
        }))
    }
    fn carry(
        &mut self,
        old: &Arc<Node<Slot<R>>>,
        value: Slot<R>,
        left: Link<Slot<R>>,
        right: Link<Slot<R>>,
    ) -> Result<Arc<Node<Slot<R>>>, ReductionRunStop<LeaseDenial>> {
        self.build(old.identity, value, left, right)
    }
}
