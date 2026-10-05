use crate::facade::{AspectVersion, NodeId, SignalError};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::DependencyBatchBuilder;

use super::super::audit_surface::PrimaryAuditSurface;
use super::super::node_families::{
    bounded, build_aggregate_sources, build_bucket_exposure_nodes, build_instrument_nodes,
    build_scenario_nodes, AggregateSourceNodes, FintechRuntime,
};
use super::super::scales::FintechScale;
use super::market_world::MarketWorld;
use super::InstrumentFixture;

pub(super) struct PortfolioWorld {
    pub(super) aggregate_sources: Vec<AggregateSourceNodes>,
    pub(super) instruments: Vec<InstrumentFixture>,
    pub(super) book_aggregates: Vec<NodeId>,
    pub(super) desk_aggregates: Vec<NodeId>,
}

pub(in crate::tests::domains::fintech) fn build_portfolio_world(
    runtime: &mut FintechRuntime,
    scale: FintechScale,
    market: &MarketWorld,
) -> PortfolioWorld {
    let fx = market.fx;
    let scenario_sources = market.scenario_sources.as_slice();
    let mut aggregate_sources = Vec::with_capacity(scale.books.max(scale.desks));
    for _ in 0..scale.books.max(scale.desks) {
        aggregate_sources.push(build_aggregate_sources(runtime));
    }

    let mut instruments = Vec::with_capacity(scale.instruments);
    for instrument_index in 0..scale.instruments {
        let core = build_instrument_nodes(runtime);
        let buckets = build_bucket_exposure_nodes(
            runtime,
            &core,
            &market.curve_buckets,
            &market.vol_surface_buckets,
        );
        let scenarios = build_scenario_nodes(runtime, &core, scenario_sources, scale.scenarios);
        instruments.push(InstrumentFixture {
            instrument_index,
            book_index: super::super::hierarchy::book_for_instrument(scale, instrument_index),
            core,
            buckets,
            scenarios,
        });
    }

    let mut book_aggregates = Vec::with_capacity(scale.books);
    for (book_index, aggregate_source) in aggregate_sources.iter().take(scale.books).enumerate() {
        let mut maximum = vec![
            (aggregate_source.book_state, super::super::aspects::RISK),
            (aggregate_source.book_state, super::super::aspects::ALERT),
            (fx.eur_jpy, super::super::aspects::PRICE),
        ];
        for instrument in &instruments {
            if instrument.book_index == book_index {
                maximum.push((instrument.core.risk, super::super::aspects::RISK));
                maximum.push((instrument.core.alert, super::super::aspects::ALERT));
            }
        }
        let aggregate = runtime
            .graph_mut()
            .node()
            .with_contract(bounded(maximum))
            .reads_aspects(super::super::aspects::full_mask())
            .tolerance(5)
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(
                aggregate,
                aggregate_source.book_state,
                super::super::aspects::RISK,
            )
            .unwrap()
            .append_dependency(
                aggregate,
                aggregate_source.book_state,
                super::super::aspects::ALERT,
            )
            .unwrap()
            .append_dependency(aggregate, fx.eur_jpy, super::super::aspects::PRICE)
            .unwrap();
        for instrument in &instruments {
            if instrument.book_index == book_index {
                dependencies
                    .append_dependency(aggregate, instrument.core.risk, super::super::aspects::RISK)
                    .unwrap()
                    .append_dependency(
                        aggregate,
                        instrument.core.alert,
                        super::super::aspects::ALERT,
                    )
                    .unwrap();
            }
        }
        dependencies.commit().unwrap();
        book_aggregates.push(aggregate);
    }

    let mut desk_aggregates = Vec::with_capacity(scale.desks);
    for (desk_index, aggregate_source) in aggregate_sources.iter().take(scale.desks).enumerate() {
        let mut maximum = vec![
            (aggregate_source.desk_limit, super::super::aspects::RISK),
            (aggregate_source.desk_limit, super::super::aspects::ALERT),
        ];
        for (book_index, book_node) in book_aggregates.iter().enumerate() {
            if super::super::hierarchy::desk_for_book(scale, book_index) == desk_index {
                maximum.push((*book_node, super::super::aspects::RISK));
                maximum.push((*book_node, super::super::aspects::ALERT));
            }
        }
        let aggregate = runtime
            .graph_mut()
            .node()
            .with_contract(bounded(maximum))
            .reads_aspects(super::super::aspects::full_mask())
            .tolerance(6)
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(
                aggregate,
                aggregate_source.desk_limit,
                super::super::aspects::RISK,
            )
            .unwrap()
            .append_dependency(
                aggregate,
                aggregate_source.desk_limit,
                super::super::aspects::ALERT,
            )
            .unwrap();
        for (book_index, book_node) in book_aggregates.iter().enumerate() {
            if super::super::hierarchy::desk_for_book(scale, book_index) == desk_index {
                dependencies
                    .append_dependency(aggregate, *book_node, super::super::aspects::RISK)
                    .unwrap();
            }
        }
        dependencies.commit().unwrap();
        desk_aggregates.push(aggregate);
    }
    PortfolioWorld {
        aggregate_sources,
        instruments,
        book_aggregates,
        desk_aggregates,
    }
}

impl super::FintechWorld {
    pub(in crate::tests::domains::fintech) fn top_desk(&self) -> NodeId {
        self.handles.aggregate.top_desk
    }

    pub(in crate::tests::domains::fintech) fn top_scenario(&self) -> NodeId {
        self.handles.aggregate.top_scenario
    }

    pub(in crate::tests::domains::fintech) fn read_node(
        &mut self,
        node: NodeId,
    ) -> Result<AspectVersion, SignalError> {
        let evaluation = self.evaluation_shape();
        let evaluator = evaluation.evaluator();
        self.runtime.read(node, &(), &evaluator)
    }

    pub(crate) fn read_top_desk(&mut self) -> Result<AspectVersion, SignalError> {
        self.read_node(self.top_desk())
    }

    pub(crate) fn read_top_scenario(&mut self) -> Result<AspectVersion, SignalError> {
        self.read_node(self.top_scenario())
    }

    pub(in crate::tests::domains::fintech) fn read_primary_audit_surface(
        &mut self,
    ) -> Result<PrimaryAuditSurface, SignalError> {
        let desk = self.read_top_desk()?;
        let scenario = self.read_top_scenario()?;
        Ok(PrimaryAuditSurface::new(desk, scenario))
    }

    pub(in crate::tests::domains::fintech) fn read_primary_audit_surface_with_workers(
        &mut self,
        workers: usize,
    ) -> Result<PrimaryAuditSurface, SignalError> {
        let targets = [self.top_desk(), self.top_scenario()];
        let evaluation = self.evaluation_shape();
        let checked = evaluation.checked_evaluator();
        let lease = portfolio_lease(workers)?;
        for target in targets {
            self.runtime.evaluate_checked(
                &[target],
                crate::facade::EvaluationRequestMode::Default,
                &(),
                &checked,
                &lease,
            )?;
        }
        Ok(PrimaryAuditSurface::new(
            self.runtime.graph().node_aspect_version(targets[0])?,
            self.runtime.graph().node_aspect_version(targets[1])?,
        ))
    }

    pub(in crate::tests::domains::fintech) fn refresh_primary_audit_surface(
        &mut self,
    ) -> Result<PrimaryAuditSurface, SignalError> {
        let top_desk = self.top_desk();
        let top_scenario = self.top_scenario();
        let evaluation = self.evaluation_shape();
        let evaluator = evaluation.evaluator();
        self.runtime.transaction(&mut (), |tx| {
            tx.read(top_desk, &evaluator)?;
            tx.read(top_scenario, &evaluator)?;
            Ok(())
        })?;
        self.read_primary_audit_surface()
    }

    pub(in crate::tests::domains::fintech) fn refresh_primary_audit_surface_with_workers(
        &mut self,
        workers: usize,
    ) -> Result<PrimaryAuditSurface, SignalError> {
        let top_desk = self.top_desk();
        let top_scenario = self.top_scenario();
        let evaluation = self.evaluation_shape();
        let checked = evaluation.checked_evaluator();
        let lease = portfolio_lease(workers)?;
        self.runtime.transaction(&mut (), |tx| {
            tx.evaluate_checked(
                &[top_desk],
                crate::facade::EvaluationRequestMode::Default,
                &checked,
                &lease,
            )?;
            tx.evaluate_checked(
                &[top_scenario],
                crate::facade::EvaluationRequestMode::Default,
                &checked,
                &lease,
            )?;
            Ok(())
        })?;
        self.read_primary_audit_surface_with_workers(workers)
    }
}

fn portfolio_lease(
    workers: usize,
) -> Result<worth_execution::ExecutionResourceLease<'static>, SignalError> {
    // Each checked evaluation is a real request under the shared host. The
    // two target requests keep their caller order across worker settings.
    let mut admission = request(workers, 100_000_000);
    admission.policy = worth_foundational::ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        worth_foundational::ExecutionBudget::new(
            admission.policy.budget().max_workers(),
            64 * 1024 * 1024,
            100_000_000,
        ),
    );
    authority()
        .request_lease(admission)
        .map_err(|_| SignalError::invalid_input("fintech workflow lease denied"))
}
