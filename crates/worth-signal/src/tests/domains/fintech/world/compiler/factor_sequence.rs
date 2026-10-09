use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::facade::NodeState;

use super::super::{FinancialWorldDefinition, InstrumentId, MarketFactorKey, SemanticOutputKey};
use super::evaluation::FinancialEvaluationProgram;
use super::runtime_finance::runtime_financial_snapshot;
use super::topology::factor_signal_aspect;
use super::{source_result, CompiledFinancialWorld};

pub(in crate::tests::domains::fintech) struct FinancialFactorSequenceEvidence {
    pending_scopes: Vec<PartitionSubscription>,
    gated_consumer_was_pending: bool,
}

pub(in crate::tests::domains::fintech) struct FinancialGatedSequenceEvidence {
    baseline_revision: u64,
    final_revision: u64,
}

impl FinancialGatedSequenceEvidence {
    pub(in crate::tests::domains::fintech) const fn revision_delta(&self) -> u64 {
        self.final_revision.abs_diff(self.baseline_revision)
    }
}

impl FinancialFactorSequenceEvidence {
    pub(in crate::tests::domains::fintech) fn pending_scopes(&self) -> &[PartitionSubscription] {
        &self.pending_scopes
    }

    pub(in crate::tests::domains::fintech) const fn gated_consumer_was_pending(&self) -> bool {
        self.gated_consumer_was_pending
    }
}

impl CompiledFinancialWorld {
    pub(in crate::tests::domains::fintech) fn apply_factor_change_sequence(
        &mut self,
        changes: &[(FinancialWorldDefinition, MarketFactorKey)],
        affected_instrument: InstrumentId,
    ) -> Result<FinancialFactorSequenceEvidence, SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        self.ledger.clear();
        for (next_definition, factor) in changes {
            self.commit_factor_source_change(next_definition, *factor)?;
        }

        let valuation = self.handles.position(affected_instrument).valuation;
        let pending_scopes = self
            .runtime
            .graph()
            .pending_causes(valuation)?
            .iter()
            .flat_map(|cause| cause.changed_scopes.iter().cloned())
            .collect::<Vec<_>>();
        let program = self.program();
        let evaluator = program.evaluator();
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
            .handles
            .consumers
            .values()
            .map(|handle| handle.0)
            .collect::<Vec<_>>();
        for wave in [valuation_wave, risk_wave] {
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
        let gated_consumer_was_pending = consumer_wave.iter().any(|consumer| {
            self.runtime
                .graph()
                .pending_causes(*consumer)
                .is_ok_and(|causes| !causes.is_empty())
                || self
                    .runtime
                    .graph()
                    .pending_dependency_revalidation(*consumer)
                    .ok()
                    .flatten()
                    .is_some_and(|pending| !pending.is_resolved())
        });
        self.runtime.transaction(request_execution, &mut (), |tx| {
            for consumer in &consumer_wave {
                tx.read(*consumer, &evaluator)?;
            }
            Ok(())
        })?;
        Ok(FinancialFactorSequenceEvidence {
            pending_scopes,
            gated_consumer_was_pending,
        })
    }

    fn commit_factor_source_change(
        &mut self,
        next_definition: &FinancialWorldDefinition,
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

        let next_snapshot = runtime_financial_snapshot(next_definition);
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
            tx.mark_changed(source, factor_signal_aspect(next_definition, factor))?;
            ledger.record(SemanticOutputKey::Factor(factor));
            tx.target(source)
                .on_demand()
                .read(&move |view| Ok(view.finish(result.clone())))?;
            Ok(())
        })?;
        self.definition = next_definition.clone();
        self.economic_snapshot = next_snapshot;
        self.projection = next_projection;
        Ok(())
    }

    pub(in crate::tests::domains::fintech) fn apply_gated_factor_sequence(
        &mut self,
        changes: &[(FinancialWorldDefinition, MarketFactorKey)],
        affected_instrument: InstrumentId,
        consumer_role: super::super::FinancialConsumerRole,
    ) -> Result<FinancialGatedSequenceEvidence, SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let risk_key = SemanticOutputKey::Risk(affected_instrument);
        let baseline_revision = self.projection.output(risk_key).revision;
        self.ledger.clear();
        for (next_definition, factor) in changes {
            let next_snapshot = runtime_financial_snapshot(next_definition);
            let next_projection = self.projection.advance(&next_snapshot);
            let program = FinancialEvaluationProgram::new(
                next_definition.clone(),
                next_projection.clone(),
                self.handles.clone(),
                self.ledger.clone(),
            );
            let evaluator = program.evaluator();
            let source = self.handles.factor(*factor).0;
            let result = source_result(&program, *factor);
            let ledger = self.ledger.clone();
            self.runtime.transaction(request_execution, &mut (), |tx| {
                tx.mark_changed(source, factor_signal_aspect(next_definition, *factor))?;
                ledger.record(SemanticOutputKey::Factor(*factor));
                tx.target(source)
                    .on_demand()
                    .read(&move |view| Ok(view.finish(result.clone())))?;
                Ok(())
            })?;
            let valuation_wave = next_definition
                .positions()
                .iter()
                .map(|position| self.handles.position(position.instrument).valuation)
                .collect::<Vec<_>>();
            let risk_wave = next_definition
                .positions()
                .iter()
                .map(|position| self.handles.position(position.instrument).risk)
                .collect::<Vec<_>>();
            for wave in [valuation_wave, risk_wave] {
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
            self.definition = next_definition.clone();
            self.economic_snapshot = next_snapshot;
            self.projection = next_projection;
        }
        let program = self.program();
        let evaluator = program.evaluator();
        let consumer = self.handles.consumer(consumer_role).0;
        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.read(consumer, &evaluator)?;
            Ok(())
        })?;
        Ok(FinancialGatedSequenceEvidence {
            baseline_revision,
            final_revision: self.projection.output(risk_key).revision,
        })
    }
}
