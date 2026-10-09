use std::collections::BTreeMap;

use crate::data::error::SignalError;
use crate::data::output::{ChangedRegion, NodeEvaluationResult};
use crate::facade::{NodeState, SignalGraph, SignalRuntimePolicy};

use super::super::super::baseline::{seal_financial_baseline, CausallyCompleteFinancialBaseline};
use super::super::super::{
    FinancialSemanticProjection, FinancialWorldDefinition, MarketFactorKey, SemanticOutputKey,
};
use super::super::evaluation::FinancialEvaluationProgram;
use super::super::runtime_finance::runtime_financial_snapshot;
use super::super::topology::{build_semantic_topology, factor_signal_aspect};
use super::super::FinancialEvaluationLedger;
use super::{CompiledFinancialWorld, CompiledFinancialWorldKind, CompiledPortfolioFinancialWorld};

pub(crate) fn compile_financial_world(
    definition: FinancialWorldDefinition,
) -> Result<CausallyCompleteFinancialBaseline, SignalError> {
    compile_financial_world_with_policy(
        definition,
        SignalRuntimePolicy::fintech()
            .with_history_limit(8)
            .with_detail_limit(4),
    )
}

pub(crate) fn compile_financial_world_with_policy(
    definition: FinancialWorldDefinition,
    policy: SignalRuntimePolicy,
) -> Result<CausallyCompleteFinancialBaseline, SignalError> {
    if definition.locality().is_some() {
        return Err(SignalError::invalid_input(
            "locality courtrooms require compile_financial_locality_world",
        ));
    }
    let mut runtime = crate::facade::SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_tiers::<crate::tests::domains::fintech::execution_tier::FintechTier>()
        .runtime_policy(policy)
        .build();
    let handles = build_semantic_topology(&mut runtime.graph_mut(), &definition)?;
    let economic_snapshot = runtime_financial_snapshot(&definition);
    let projection = FinancialSemanticProjection::initial(&economic_snapshot);
    let ledger = FinancialEvaluationLedger::default();
    let baseline_dependency_revisions = projection
        .iter()
        .map(|(key, _)| {
            Ok((
                key,
                runtime
                    .graph()
                    .dependency_revision(handles.node_for(key))?
                    .0,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, SignalError>>()?;
    let portfolio = CompiledPortfolioFinancialWorld {
        runtime,
        definition,
        economic_snapshot,
        projection,
        handles,
        ledger,
        baseline_dependency_revisions,
        baseline_aspect_versions: BTreeMap::new(),
    };
    let mut compiled = CompiledFinancialWorld {
        kind: CompiledFinancialWorldKind::Portfolio(portfolio),
    };
    compiled.establish_initial_truth()?;
    compiled.baseline_aspect_versions = compiled
        .projection
        .iter()
        .map(|(key, _)| Ok((key, compiled.node_version(key)?)))
        .collect::<Result<BTreeMap<_, _>, SignalError>>()?;
    seal_financial_baseline(compiled)
}

impl CompiledFinancialWorld {
    pub(in crate::tests::domains::fintech::world::compiler) fn program(
        &self,
    ) -> FinancialEvaluationProgram {
        FinancialEvaluationProgram::new(
            self.definition.clone(),
            self.projection.clone(),
            self.handles.clone(),
            self.ledger.clone(),
        )
    }

    fn establish_initial_truth(&mut self) -> Result<(), SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let program = self.program();
        let evaluator = program.evaluator();
        let factor_nodes = self
            .handles
            .factors
            .iter()
            .map(|(factor, handle)| (*factor, handle.0))
            .collect::<Vec<_>>();
        let risk_nodes = self
            .definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).risk)
            .collect::<Vec<_>>();
        let consumer_nodes = self
            .definition
            .consumers()
            .iter()
            .map(|consumer| self.handles.consumer(consumer.role).0)
            .collect::<Vec<_>>();
        self.runtime.transaction(request_execution, &mut (), |tx| {
            for (factor, node) in &factor_nodes {
                let result = source_result(&program, *factor);
                tx.target(*node)
                    .on_demand()
                    .read(&move |view| Ok(view.finish(result.clone())))?;
            }
            for node in &risk_nodes {
                tx.read(*node, &evaluator)?;
            }
            for node in &consumer_nodes {
                tx.read(*node, &evaluator)?;
            }
            Ok(())
        })?;
        self.ledger.clear();
        Ok(())
    }

    pub(in crate::tests::domains::fintech) fn apply_factor_change(
        &mut self,
        next_definition: FinancialWorldDefinition,
        factor: MarketFactorKey,
    ) -> Result<(), SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let next_snapshot = runtime_financial_snapshot(&next_definition);
        let next_projection = self.projection.advance(&next_snapshot);
        let program = FinancialEvaluationProgram::new(
            next_definition.clone(),
            next_projection.clone(),
            self.handles.clone(),
            self.ledger.clone(),
        );
        let evaluator = program.evaluator();
        let source = self.handles.factor(factor).0;
        let valuation_wave = self
            .definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).valuation)
            .collect::<Vec<_>>();
        let risk_wave = self
            .definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).risk)
            .collect::<Vec<_>>();
        let consumer_wave = self
            .definition
            .consumers()
            .iter()
            .map(|consumer| self.handles.consumer(consumer.role).0)
            .collect::<Vec<_>>();
        let source_result = source_result(&program, factor);
        let ledger = self.ledger.clone();
        self.ledger.clear();
        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.mark_changed(source, factor_signal_aspect(&next_definition, factor))?;
            ledger.record(SemanticOutputKey::Factor(factor));
            tx.target(source)
                .on_demand()
                .read(&move |view| Ok(view.finish(source_result.clone())))?;
            Ok(())
        })?;
        for wave in [valuation_wave, risk_wave, consumer_wave] {
            let dirty = wave
                .into_iter()
                .filter(|node| {
                    self.runtime
                        .graph()
                        .get_state(*node)
                        .is_ok_and(|state| !matches!(state, NodeState::Clean))
                })
                .collect::<Vec<_>>();
            self.runtime.transaction(request_execution, &mut (), |tx| {
                for node in &dirty {
                    tx.read(*node, &evaluator)?;
                }
                Ok(())
            })?;
        }
        self.definition = next_definition;
        self.economic_snapshot = next_snapshot;
        self.projection = next_projection;
        Ok(())
    }

    pub(crate) fn apply_first_market_factor_change(
        &mut self,
        next_definition: FinancialWorldDefinition,
    ) -> Result<(), SignalError> {
        let factor = self.definition.first_market_factor();
        self.apply_factor_change(next_definition, factor)
    }
}

pub(in crate::tests::domains::fintech::world::compiler) fn source_result(
    program: &FinancialEvaluationProgram,
    factor: MarketFactorKey,
) -> NodeEvaluationResult {
    let (partition, detail) = factor.partition();
    program
        .result_for(SemanticOutputKey::Factor(factor))
        .with_changed_region(ChangedRegion::new(partition).with_detail(detail))
}
