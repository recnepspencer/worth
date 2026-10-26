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
impl AggregateObservation {
    pub(in crate::domain_computation::primary_graph) fn read(
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        relation: &str,
        entity_kind: KindId,
        relation_kind: KindId,
        member: EntityId,
        maximum_work: Option<u64>,
    ) -> Result<Arc<Self>, PinnedAggregateReadDenial> {
        let native = pin.read_aggregate(
            execution,
            PinnedAggregateReadScope::new(relation_kind, member).with_work_limit(maximum_work),
        )?;
        let key = AggregateKey::new(
            relation,
            entity_kind,
            relation_kind,
            member,
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
