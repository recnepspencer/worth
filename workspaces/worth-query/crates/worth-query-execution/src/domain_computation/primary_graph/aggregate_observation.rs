//! Actual selected-root aggregate truth, distinct from successor-walk evidence.
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        PinnedAggregateRead, PinnedAggregateReadDenial, PinnedAggregateReadScope,
        RelationalPinnedRead,
    },
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};
mod currentness;
mod key;
pub(in crate::domain_computation::primary_graph) use key::AggregateKey;

/// Only Native acquisition can construct this body. Source custody is separate
/// from semantic truth: stored incident lineage is not part of an orbit answer.
#[derive(Debug)]
pub(in crate::domain_computation) struct AggregateObservation {
    key: AggregateKey,
    native: PinnedAggregateRead,
    source: SnapshotHandle,
}
pub(in crate::domain_computation::primary_graph) struct AggregateQuery<'a> {
    pub relation: &'a str,
    pub entity_kind: KindId,
    pub relation_kind: KindId,
    pub member: EntityId,
    pub maximum_work: Option<u64>,
    pub control: Option<&'a dyn Fn() -> Result<(), Cause>>,
}
impl AggregateObservation {
    pub(in crate::domain_computation::primary_graph) fn read(
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        query: AggregateQuery<'_>,
    ) -> Result<Arc<Self>, PinnedAggregateReadDenial> {
        let mut scope = PinnedAggregateReadScope::new(query.relation_kind, query.member)
            .with_work_limit(query.maximum_work);
        if let Some(control) = query.control {
            scope = scope.with_control(control);
        }
        let native = pin.read_aggregate(execution, scope)?;
        let key = AggregateKey::new(
            query.relation,
            query.entity_kind,
            query.relation_kind,
            query.member,
            native.definition(),
        );
        Ok(Arc::new(Self {
            key,
            native,
            source: pin.snapshot().clone(),
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &AggregateKey {
        &self.key
    }
    pub(in crate::domain_computation::primary_graph) fn native(&self) -> &PinnedAggregateRead {
        &self.native
    }
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &SnapshotHandle {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        if self.key != other.key
            || self.native.definition() != other.native.definition()
            || self.native.identity() != other.native.identity()
            || self.native.len() != other.native.len()
        {
            return Ok(false);
        }
        for (a, b) in self.native.members().zip(other.native.members()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
    pub(in crate::domain_computation::primary_graph) fn checkpoint(
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Cause> {
        (if context.has_accounting() {
            context.checkpoint(1)
        } else {
            context.poll()
        })
        .map_err(|stop| WorkCeilingDenial::Stopped(stop).into())
    }
}
impl PartialEq for AggregateObservation {
    fn eq(&self, other: &Self) -> bool {
        self.same_truth_with_poll(other, || Ok::<(), std::convert::Infallible>(()))
            .unwrap()
    }
}
impl Eq for AggregateObservation {}
