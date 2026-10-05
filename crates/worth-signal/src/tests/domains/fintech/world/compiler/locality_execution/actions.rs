use std::collections::BTreeSet;

use crate::data::error::SignalError;
use crate::data::node::NodeState;
use crate::data::output::ChangedRegion;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::invalidation::scheduling::merge_repeated_current_admission;
use crate::logic::planner::{StageExecutionOutcome, StageExecutionRecord};
use crate::tests::leased_execution::support::{authority, request};

use super::super::locality_evaluation::runtime_shocked_values_for_batch;
use super::physical_ready::captured_bindings;
use super::{
    signal_aspect, CompiledFinancialLocalityWorld, FinancialLocalityRedObservation,
    LocalityEvaluationProgram, LocalitySemanticOutputId, PhysicalReadyWitness, RedObservationInput,
};
use crate::tests::domains::fintech::world::FinancialLocalityAction;
use crate::tests::domains::fintech::world::FinancialLocalityMutation;
use crate::tests::domains::fintech::world::FinancialLocalityScenario;

mod churn;
mod churn_program;
mod restore;

pub(in crate::tests::domains::fintech) use restore::FinancialRestoreLifecycleEvidence;

pub(super) struct LocalityExecutionSettlement {
    pub(super) evaluated_outputs: BTreeSet<LocalitySemanticOutputId>,
    pub(super) stage_outcomes: Vec<StageExecutionOutcome>,
    pub(super) stage_records: Vec<StageExecutionRecord>,
    pub(super) physical_ready: Option<PhysicalReadyWitness>,
}

impl CompiledFinancialLocalityWorld {
    pub(in crate::tests::domains::fintech::world::compiler) fn certify_restore_lifecycle(
        &mut self,
    ) -> Result<FinancialRestoreLifecycleEvidence, SignalError> {
        restore::certify_restore_lifecycle(self)
    }

    pub(in crate::tests::domains::fintech) fn run_action_trace(
        &mut self,
        trace_index: usize,
    ) -> Result<FinancialLocalityRedObservation, SignalError> {
        self.run_action_trace_with_workers(trace_index, 1)
    }

    pub(in crate::tests::domains::fintech) fn run_action_trace_with_workers(
        &mut self,
        trace_index: usize,
        workers: usize,
    ) -> Result<FinancialLocalityRedObservation, SignalError> {
        self.runtime
            .graph_mut()
            .reset_invalidation_performed_counters();
        let before = self.runtime.graph().telemetry().invalidation;
        let evaluation_before = self.runtime.graph().telemetry().evaluation;
        let trace = &self.locality_definition().action_traces()[trace_index];
        let mutations = trace.committed_mutations();
        let retry_targets = trace
            .actions()
            .iter()
            .filter_map(|action| match action {
                FinancialLocalityAction::RetryAdmission { target, .. } => Some(*target),
                _ => None,
            })
            .collect::<Vec<_>>();
        let settlement = if self.locality_definition().scenario()
            == FinancialLocalityScenario::PortfolioDependencyChurn
        {
            churn::run_churn_trace(self, trace_index, workers)?
        } else {
            self.apply_mutations(&mutations)?;
            self.settle_mutations_with_retries(&mutations, &retry_targets, workers)?
        };
        let after = self.runtime.graph().telemetry().invalidation;
        let evaluation_after = self.runtime.graph().telemetry().evaluation;
        let baseline_retained_outputs =
            self.baseline_retained_outputs(&settlement.evaluated_outputs)?;
        let graph = self.runtime.graph();
        let explanation_fact_count = self
            .handles
            .values()
            .filter(|node| graph.observe().explanation_fact(**node).is_some())
            .count();
        let provenance_fact_count = self
            .handles
            .values()
            .filter(|node| graph.observe().provenance_fact(**node).is_some())
            .count();
        Ok(self.red_observation(RedObservationInput {
            before,
            after,
            evaluation_before,
            evaluation_after,
            evaluated_outputs: settlement.evaluated_outputs,
            baseline_retained_outputs,
            performed: self.runtime.graph().invalidation_performed_counters(),
            execution_stage_outcomes: settlement.stage_outcomes,
            physical_ready: settlement.physical_ready.ok_or_else(|| {
                SignalError::internal("certification omitted its physical ready witness")
            })?,
            lineage_records: self.runtime.graph().observe().lineage_records().len(),
            explanation_fact_count,
            provenance_fact_count,
            frontier_summary_retained: graph
                .observe()
                .latest_frontier_execution_summary()
                .is_some(),
            replay_event_count: graph.observe().replay_events().len(),
            flow_summary_retained: graph.observe().latest_flow_diagnostics().is_some(),
        }))
    }

    pub(super) fn settle_mutations_with_retries(
        &mut self,
        mutations: &[FinancialLocalityMutation],
        retry_targets: &[LocalitySemanticOutputId],
        workers: usize,
    ) -> Result<LocalityExecutionSettlement, SignalError> {
        self.settle_mutations_with_retries_at_batch(mutations, retry_targets, workers, 0, true)
    }

    pub(super) fn settle_mutations_with_retries_at_batch(
        &mut self,
        mutations: &[FinancialLocalityMutation],
        retry_targets: &[LocalitySemanticOutputId],
        workers: usize,
        batch_index: usize,
        capture_physical_witness: bool,
    ) -> Result<LocalityExecutionSettlement, SignalError> {
        let shocked_values = runtime_shocked_values_for_batch(
            self.locality_definition(),
            &self.baseline_values,
            mutations,
            batch_index,
        )?;
        let program = LocalityEvaluationProgram::shocked_for_batch(
            self.locality_definition(),
            &self.handles,
            &self.baseline_values,
            &shocked_values,
            mutations,
            batch_index,
        );
        let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
        let mut physical_ready = capture_physical_witness.then(PhysicalReadyWitness::default);
        for mutation in mutations {
            let source = self.handles[&mutation.producer];
            let before = physical_ready
                .as_ref()
                .map(|_| captured_bindings(self.runtime.graph()));
            self.runtime
                .transaction(&mut (), |tx| tx.read(source, &evaluator).map(|_| ()))?;
            if let (Some(before), Some(witness)) = (before, physical_ready.as_mut()) {
                witness.record_transaction(
                    source,
                    self.runtime.graph(),
                    &before,
                    &captured_bindings(self.runtime.graph()),
                )?;
            }
        }
        for target in retry_targets {
            merge_repeated_current_admission(&mut self.runtime.graph_mut(), self.handles[target])?;
        }
        let release_waves = self
            .locality_definition()
            .workload()
            .release_waves()
            .to_vec();
        let mut stage_outcomes = Vec::new();
        let mut stage_records = Vec::new();
        for wave in release_waves {
            let nodes = wave
                .iter()
                .map(|output| self.handles[output])
                .filter(|node| {
                    self.runtime
                        .graph()
                        .get_state(*node)
                        .is_ok_and(|state| !matches!(state, NodeState::Clean))
                })
                .collect::<Vec<_>>();
            if nodes.is_empty() {
                continue;
            }
            // Dense worlds include planning and retained evidence in the same
            // finite request; worker posture does not change that allowance.
            let mut admission = request(workers, 100_000_000);
            admission.policy = worth_foundational::ExecutionRequestPolicy::new(
                admission.policy.posture(),
                admission.policy.determinism(),
                worth_foundational::ExecutionBudget::new(
                    admission.policy.budget().max_workers(),
                    128 * 1024 * 1024,
                    100_000_000,
                ),
            );
            let lease = authority()
                .request_lease(admission)
                .map_err(|_| SignalError::invalid_input("locality host lease denied"))?;
            let before = physical_ready
                .as_ref()
                .map(|_| captured_bindings(self.runtime.graph()));
            let report = self.runtime.evaluate_checked(
                &nodes,
                EvaluationRequestMode::Default,
                &(),
                &|view| program.evaluate_checked(view),
                &lease,
            )?;
            if let (Some(before), Some(witness)) = (before, physical_ready.as_mut()) {
                witness.record_checked(
                    &before,
                    &captured_bindings(self.runtime.graph()),
                    &report.stages,
                )?;
            }
            program.record_completed(&report);
            for stage in report.stages {
                stage_outcomes.push(stage.outcome);
                stage_records.push(stage);
            }
        }
        Ok(LocalityExecutionSettlement {
            evaluated_outputs: program.evaluated_outputs(),
            stage_outcomes,
            stage_records,
            physical_ready,
        })
    }

    pub(super) fn apply_mutations(
        &mut self,
        mutations: &[FinancialLocalityMutation],
    ) -> Result<(), SignalError> {
        self.runtime.transaction(&mut (), |tx| {
            let mut batch = tx.batch_changes();
            for mutation in mutations {
                let source = self.handles[&mutation.producer];
                let aspect = signal_aspect(mutation.aspect);
                batch = match mutation.scope.map(|scope| {
                    let mut region = ChangedRegion::new(scope.partition_label());
                    if let Some(detail) = scope.detail_label() {
                        region = region.with_detail(detail);
                    }
                    region
                }) {
                    None => batch.mark(source, aspect),
                    Some(region) => {
                        batch.mark_regions(source, aspect, std::slice::from_ref(&region))
                    }
                };
            }
            batch.apply().map(|_| ())
        })?;
        Ok(())
    }
}
