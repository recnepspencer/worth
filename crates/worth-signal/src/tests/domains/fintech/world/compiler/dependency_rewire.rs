use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::facade::NodeState;

use super::super::{FinancialWorldDefinition, InstrumentId, MarketFactorKey, SemanticOutputKey};
use super::evaluation::FinancialEvaluationProgram;
use super::runtime_finance::runtime_financial_snapshot;
use super::topology::factor_signal_aspect;
use super::{source_result, CompiledFinancialWorld};

pub(in crate::tests::domains::fintech) struct FinancialDependencyRewireEvidence {
    topology_owner: SemanticOutputKey,
    baseline_revision: u64,
    stale_revision: u64,
    final_revision: u64,
    stale_cause_rejected: bool,
    cycle_rejected: bool,
}

struct StaleRewirePosture {
    valuation: crate::data::handle::NodeId,
    risk: crate::data::handle::NodeId,
    topology_owner: SemanticOutputKey,
    baseline_revision: u64,
    stale_revision: u64,
    stale_cause_rejected: bool,
}

impl FinancialDependencyRewireEvidence {
    pub(in crate::tests::domains::fintech) const fn topology_owner(&self) -> SemanticOutputKey {
        self.topology_owner
    }

    pub(in crate::tests::domains::fintech) const fn baseline_revision(&self) -> u64 {
        self.baseline_revision
    }

    pub(in crate::tests::domains::fintech) const fn stale_revision(&self) -> u64 {
        self.stale_revision
    }

    pub(in crate::tests::domains::fintech) const fn final_revision(&self) -> u64 {
        self.final_revision
    }

    pub(in crate::tests::domains::fintech) const fn stale_cause_rejected(&self) -> bool {
        self.stale_cause_rejected
    }

    pub(in crate::tests::domains::fintech) const fn cycle_rejected(&self) -> bool {
        self.cycle_rejected
    }
}

impl CompiledFinancialWorld {
    pub(in crate::tests::domains::fintech) fn apply_instrument_dependency_rewire(
        &mut self,
        cause_definition: FinancialWorldDefinition,
        cause_factor: MarketFactorKey,
        final_definition: FinancialWorldDefinition,
        instrument: InstrumentId,
    ) -> Result<FinancialDependencyRewireEvidence, SignalError> {
        self.stage_factor_change(cause_definition, cause_factor)?;
        let stale = self.establish_stale_rewire_posture(instrument)?;
        let (final_revision, cycle_rejected) =
            self.apply_current_rewire(&final_definition, instrument, stale.valuation, stale.risk)?;
        self.settle_current_rewire(final_definition)?;
        Ok(FinancialDependencyRewireEvidence {
            topology_owner: stale.topology_owner,
            baseline_revision: stale.baseline_revision,
            stale_revision: stale.stale_revision,
            final_revision,
            stale_cause_rejected: stale.stale_cause_rejected,
            cycle_rejected,
        })
    }

    fn establish_stale_rewire_posture(
        &mut self,
        instrument: InstrumentId,
    ) -> Result<StaleRewirePosture, SignalError> {
        let valuation = self.handles.position(instrument).valuation;
        let risk = self.handles.position(instrument).risk;
        let topology_owner = SemanticOutputKey::Valuation(instrument);
        let baseline_revision = self.baseline_dependency_revision(topology_owner);
        let stale = self
            .runtime
            .graph()
            .pending_causes(valuation)?
            .first()
            .cloned()
            .ok_or_else(|| {
                SignalError::internal("rewire courtroom failed to stage an old cause")
            })?;
        let stale_revision = stale.key.dependency_revision.0;
        let old_edges = self.runtime.graph().dependencies_of(valuation)?.to_vec();
        self.runtime.graph_mut().clear_dependencies(valuation)?;
        self.runtime
            .graph_mut()
            .set_dependencies(valuation, old_edges)?;
        let stale_cause_rejected = self
            .runtime
            .graph_mut()
            .replace_pending_causes(valuation, [stale])
            .is_err();
        Ok(StaleRewirePosture {
            valuation,
            risk,
            topology_owner,
            baseline_revision,
            stale_revision,
            stale_cause_rejected,
        })
    }

    fn apply_current_rewire(
        &mut self,
        final_definition: &FinancialWorldDefinition,
        instrument: InstrumentId,
        valuation: crate::data::handle::NodeId,
        risk: crate::data::handle::NodeId,
    ) -> Result<(u64, bool), SignalError> {
        let next_edges = final_definition
            .position(instrument)
            .subscriptions
            .iter()
            .map(|subscription| {
                DependencyEdge::partition_detail(
                    self.handles.factor(subscription.factor).0,
                    factor_signal_aspect(final_definition, subscription.factor),
                    subscription.partition,
                    subscription.detail,
                )
            })
            .collect::<Vec<_>>();
        self.runtime
            .graph_mut()
            .set_dependencies(valuation, next_edges)?;
        let final_revision = self.runtime.graph().dependency_revision(valuation)?.0;
        let new_factor = final_definition
            .position(instrument)
            .subscriptions
            .iter()
            .find(|subscription| {
                !self
                    .definition
                    .position(instrument)
                    .subscriptions
                    .iter()
                    .any(|old| old.factor == subscription.factor)
            })
            .map(|subscription| subscription.factor)
            .ok_or_else(|| SignalError::internal("rewire courtroom lacks a new factor"))?;
        let new_source = self.handles.factor(new_factor).0;
        let cycle_result = self.runtime.graph_mut().set_dependencies(
            new_source,
            [DependencyEdge::new(
                risk,
                factor_signal_aspect(final_definition, new_factor),
            )],
        );
        let cycle_rejected =
            cycle_result.is_err() && self.runtime.graph().dependencies_of(new_source)?.is_empty();
        Ok((final_revision, cycle_rejected))
    }

    fn settle_current_rewire(
        &mut self,
        final_definition: FinancialWorldDefinition,
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

        let next_snapshot = runtime_financial_snapshot(&final_definition);
        let next_projection = self.projection.advance(&next_snapshot);
        let program = FinancialEvaluationProgram::new(
            final_definition.clone(),
            next_projection.clone(),
            self.handles.clone(),
            self.ledger.clone(),
        );
        let evaluator = program.evaluator();
        let valuation_wave = final_definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).valuation)
            .collect::<Vec<_>>();
        let risk_wave = final_definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).risk)
            .collect::<Vec<_>>();
        let consumer_wave = self
            .handles
            .consumers
            .values()
            .map(|handle| handle.0)
            .collect::<Vec<_>>();
        let staged_work = self.ledger.observed_work();
        self.ledger.clear();
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
        for key in staged_work {
            self.ledger.record(key);
        }
        self.definition = final_definition;
        self.economic_snapshot = next_snapshot;
        self.projection = next_projection;
        Ok(())
    }

    pub(super) fn stage_factor_change(
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
        let source = self.handles.factor(factor).0;
        let result = source_result(&program, factor);
        let ledger = self.ledger.clone();
        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.mark_changed(source, factor_signal_aspect(&next_definition, factor))?;
            ledger.record(SemanticOutputKey::Factor(factor));
            tx.target(source)
                .on_demand()
                .read(&move |view| Ok(view.finish(result.clone())))?;
            Ok(())
        })?;
        self.definition = next_definition;
        self.economic_snapshot = next_snapshot;
        self.projection = next_projection;
        Ok(())
    }
}
