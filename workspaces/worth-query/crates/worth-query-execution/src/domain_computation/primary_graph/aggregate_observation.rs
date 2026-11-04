//! Actual selected-root aggregate truth, distinct from successor-walk evidence.
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{PinnedAggregateRead, PinnedAggregateReadDenial, RelationalPinnedRead},
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};
mod currentness;
pub(crate) mod key;
mod member;
pub(in crate::domain_computation::primary_graph) mod member_work;
pub(crate) mod selection;
pub(in crate::domain_computation::primary_graph) use key::AggregateKey;
pub(in crate::domain_computation::primary_graph) use selection::AggregateSelection;

/// Only Native acquisition can construct this body. Source custody is separate
/// from semantic truth: stored incident lineage is not part of an orbit answer.
#[derive(Debug)]
pub struct AggregateObservation<Member = EntityId> {
    key: AggregateKey<Member>,
    native: PinnedAggregateRead<Member>,
    source: super::projection_read_origin::ProjectionReadOrigin,
    _allocation: Option<Arc<super::source_facts::SourceFactsAllocation>>,
}
pub struct AggregateQuery<'a> {
    pub allocation: Option<&'a Arc<super::source_facts::SourceFactsAllocation>>,
    pub subject: &'a str,
    pub entity_kind: KindId,
    pub selection: AggregateSelection,
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
        Self::read_projected(pin, crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef::Committed(pin.snapshot()), execution, query)
    }
    pub(in crate::domain_computation::primary_graph) fn read_projected<Phase>(
        pin: &RelationalPinnedRead<Phase>,
        origin: crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef<'_>,
        execution: &ExecutionRequest<'_, '_>,
        query: AggregateQuery<'_>,
    ) -> Result<Arc<Self>, PinnedAggregateReadDenial> {
        let mut scope = query
            .selection
            .scope(query.member)
            .with_work_limit(query.maximum_work);
        if let Some(control) = query.control {
            scope = scope.with_control(control);
        }
        let native = pin.read_aggregate(execution, scope)?;
        let key = AggregateKey::new(
            query.subject,
            query.entity_kind,
            query.selection,
            query.member,
            native.definition(),
        );
        Ok(Arc::new(Self {
            key,
            native,
            source: origin.retained(),
            _allocation: query.allocation.cloned(),
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn refresh_at(
        &self,
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, PinnedAggregateReadDenial> {
        Self::read(
            pin,
            execution,
            AggregateQuery {
                allocation: self._allocation.as_ref(),
                subject: self.key.subject(),
                entity_kind: self.key.entity_kind(),
                selection: self.key.selection,
                member: self.key.member(),
                maximum_work: None,
                control: None,
            },
        )
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

impl<Member> AggregateObservation<Member> {
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &AggregateKey<Member> {
        &self.key
    }
    pub fn native(&self) -> &PinnedAggregateRead<Member> {
        &self.native
    }
    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> &super::projection_read_origin::ProjectionReadOrigin {
        &self.source
    }
}

mod court_doors_for_test;
