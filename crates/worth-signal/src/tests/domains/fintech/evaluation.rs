mod checked_result_capacity;
mod instrument_behavior;
mod partition_behavior;
mod portfolio_aggregation;

pub(super) use checked_result_capacity::maximum_checked_result_heap_bytes;

use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::facade::{Aspect, AspectVersion, NodeEvaluationResult, PartitionSubscription};
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationOutput;

use super::node_families::{AggregateSourceNodes, FxNodes, InstrumentNodes};
use super::partition_surface::PartitionSurfaceNodes;

#[derive(Clone)]
pub(super) struct FintechEvaluationShape {
    fx: FxNodes,
    aggregate_sources: Vec<AggregateSourceNodes>,
    curve_buckets: Vec<NodeId>,
    vol_surface_buckets: Vec<NodeId>,
    scenario_sources: Vec<NodeId>,
    instruments: Vec<InstrumentShape>,
    book_aggregates: Vec<NodeId>,
    desk_aggregates: Vec<NodeId>,
    scenario_aggregates: Vec<NodeId>,
    bucket_aggregates: Vec<NodeId>,
    partition: PartitionSurfaceNodes,
}

#[derive(Clone)]
struct InstrumentShape {
    book_index: usize,
    core: InstrumentNodes,
    buckets: Vec<NodeId>,
    scenarios: Vec<NodeId>,
}

pub(super) trait FintechEvaluationView {
    fn node(&self) -> NodeId;
    fn read_aspect_version(
        &mut self,
        source: NodeId,
        aspect: Aspect,
    ) -> Result<AspectVersion, SignalError>;
    fn read_partitioned_aspect_version(
        &mut self,
        source: NodeId,
        aspect: Aspect,
        scope: PartitionSubscription,
    ) -> Result<AspectVersion, SignalError>;
    fn finish(&self, result: NodeEvaluationResult) -> EvaluationOutput;
}

impl FintechEvaluationView for EvaluationContext<'_, ()> {
    fn node(&self) -> NodeId {
        self.node()
    }
    fn read_aspect_version(
        &mut self,
        source: NodeId,
        aspect: Aspect,
    ) -> Result<AspectVersion, SignalError> {
        self.read_aspect_version(source, aspect)
    }
    fn read_partitioned_aspect_version(
        &mut self,
        source: NodeId,
        aspect: Aspect,
        scope: PartitionSubscription,
    ) -> Result<AspectVersion, SignalError> {
        self.read_partitioned_aspect_version(source, aspect, scope)
    }
    fn finish(&self, result: NodeEvaluationResult) -> EvaluationOutput {
        self.finish(result)
    }
}

impl FintechEvaluationView for CheckedEvaluationContext<'_, '_, '_, '_, ()> {
    fn node(&self) -> NodeId {
        self.node()
    }
    fn read_aspect_version(
        &mut self,
        source: NodeId,
        aspect: Aspect,
    ) -> Result<AspectVersion, SignalError> {
        Ok(AspectVersion::zero().with(aspect, self.read(source, aspect)?))
    }
    fn read_partitioned_aspect_version(
        &mut self,
        source: NodeId,
        aspect: Aspect,
        scope: PartitionSubscription,
    ) -> Result<AspectVersion, SignalError> {
        Ok(AspectVersion::zero().with(aspect, self.read_scoped(source, aspect, &scope)?))
    }
    fn finish(&self, result: NodeEvaluationResult) -> EvaluationOutput {
        self.finish(result)
    }
}

impl FintechEvaluationShape {
    pub(super) fn from_parts(
        fx: FxNodes,
        aggregate_sources: &[AggregateSourceNodes],
        curve_buckets: &[NodeId],
        vol_surface_buckets: &[NodeId],
        scenario_sources: &[NodeId],
        instruments: &[super::fixture::InstrumentFixture],
        book_aggregates: &[NodeId],
        desk_aggregates: &[NodeId],
        scenario_aggregates: &[NodeId],
        bucket_aggregates: &[NodeId],
        partition: PartitionSurfaceNodes,
    ) -> Self {
        Self {
            fx,
            aggregate_sources: aggregate_sources.to_vec(),
            curve_buckets: curve_buckets.to_vec(),
            vol_surface_buckets: vol_surface_buckets.to_vec(),
            scenario_sources: scenario_sources.to_vec(),
            instruments: instruments
                .iter()
                .map(|instrument| InstrumentShape {
                    book_index: instrument.book_index,
                    core: instrument.core,
                    buckets: instrument.buckets.clone(),
                    scenarios: instrument.scenarios.clone(),
                })
                .collect(),
            book_aggregates: book_aggregates.to_vec(),
            desk_aggregates: desk_aggregates.to_vec(),
            scenario_aggregates: scenario_aggregates.to_vec(),
            bucket_aggregates: bucket_aggregates.to_vec(),
            partition,
        }
    }

    pub(super) fn evaluator(
        &self,
    ) -> impl for<'ctx> Fn(&mut EvaluationContext<'ctx, ()>) -> Result<EvaluationOutput, SignalError>
           + Sync
           + '_ {
        move |ctx| self.evaluate_node(ctx)
    }

    pub(super) fn checked_evaluator(
        &self,
    ) -> impl for<'graph, 'work, 'run, 'lease> Fn(
        &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, ()>,
    ) -> Result<EvaluationOutput, SignalError>
           + Sync
           + '_ {
        move |ctx| {
            ctx.work()
                .checkpoint(self.instruments.len().saturating_mul(8).saturating_add(256) as u64)
                .map_err(|_| SignalError::invalid_input("fintech kernel work exhausted"))?;
            self.evaluate_node(ctx)
        }
    }

    fn evaluate_node<V: FintechEvaluationView>(
        &self,
        view: &mut V,
    ) -> Result<EvaluationOutput, SignalError> {
        if let Some(output) = self.evaluate_instrument_node(view)? {
            return Ok(output);
        }
        if let Some(output) = self.evaluate_portfolio_aggregation(view)? {
            return Ok(output);
        }
        if let Some(output) = self.evaluate_partition_node(view)? {
            return Ok(output);
        }
        Err(SignalError::invalid_input(format!(
            "unexpected fintech node {}",
            view.node()
        )))
    }
}
