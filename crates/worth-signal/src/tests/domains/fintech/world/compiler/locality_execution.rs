use std::collections::BTreeMap;

mod actions;
mod baseline;
mod compilation;
mod lifecycle_boundaries;
#[cfg(test)]
mod lifecycle_tests;
mod operational_digest;
mod optional_inventory;
mod performance;
mod performed_work;
#[cfg(test)]
mod receipt_identity_tests;
#[cfg(test)]
mod receipt_tests;
mod red_observation;
#[cfg(test)]
mod surface_selection_tests;
mod world_access;

pub(in crate::tests::domains::fintech) use actions::FinancialRestoreLifecycleEvidence;
pub(crate) use compilation::{
    compile_financial_locality_world, compile_financial_locality_world_at_tier,
    compile_financial_locality_world_with_policy,
};
pub(crate) use optional_inventory::LocalityOptionalObservationInventory;
pub(crate) use performance::FinancialPerformanceBatchReport;
#[cfg(feature = "parallel")]
pub(in crate::tests::domains::fintech) use performed_work::strategy_work_projection;
pub(in crate::tests::domains::fintech) use performed_work::{
    FinancialPerformedCanonicalWork, FinancialPerformedWorkOrigin,
};
pub(in crate::tests::domains::fintech) use red_observation::FinancialLocalityRedObservation;
use red_observation::{lineage_delta, RedObservationInput};

use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::facade::{SignalRuntime, SignalRuntimePolicy};
use crate::logic::context::EvaluationContext;
use crate::tests::domains::fintech::execution_tier::FintechTier;

use super::super::{
    FinancialLocalityDefinition, FinancialWorldDefinition, LocalitySemanticOutputId,
};
use super::locality_evaluation::LocalityEvaluationProgram;
use super::topology::signal_aspect;

type LocalityRuntime = SignalRuntime<(), (), (), (), FintechTier>;

pub(in crate::tests::domains::fintech) struct CompiledFinancialLocalityWorld {
    runtime: LocalityRuntime,
    definition: FinancialWorldDefinition,
    handles: BTreeMap<LocalitySemanticOutputId, NodeId>,
    baseline_values: BTreeMap<LocalitySemanticOutputId, i64>,
}

impl CompiledFinancialLocalityWorld {
    pub(in crate::tests::domains::fintech) fn runtime_policy(&self) -> SignalRuntimePolicy {
        self.runtime.graph().runtime_policy()
    }

    fn graph_instance(&self) -> u64 {
        self.runtime.graph().runtime_instance_id()
    }

    fn establish_causally_complete_baseline(&mut self) -> Result<(), SignalError> {
        let program = LocalityEvaluationProgram::baseline(
            self.locality_definition(),
            &self.handles,
            &self.baseline_values,
        );
        let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
        let nodes = self
            .locality_definition()
            .outputs()
            .iter()
            .map(|output| self.handles[&output.id])
            .collect::<Vec<_>>();
        self.runtime.read_many(&nodes, &(), &evaluator).map(|_| ())
    }

    pub(in crate::tests::domains::fintech) fn run_inherited_breadth_red_control(
        &mut self,
    ) -> Result<FinancialLocalityRedObservation, SignalError> {
        self.runtime
            .graph_mut()
            .reset_invalidation_performed_counters();
        let before = self.runtime.graph().telemetry().invalidation;
        let evaluation_before = self.runtime.graph().telemetry().evaluation;
        let lineage_before = self
            .runtime
            .graph()
            .observe()
            .lineage_records()
            .last()
            .map(|record| record.sequence);
        self.apply_declared_mutation()?;
        let evaluated_outputs = self.settle_declared_mutation()?;
        let after = self.runtime.graph().telemetry().invalidation;
        let evaluation_after = self.runtime.graph().telemetry().evaluation;
        let lineage_after = self
            .runtime
            .graph()
            .observe()
            .lineage_records()
            .last()
            .map(|record| record.sequence);
        let baseline_retained_outputs = self.baseline_retained_outputs(&evaluated_outputs)?;
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
            evaluated_outputs,
            baseline_retained_outputs,
            performed: self.runtime.graph().invalidation_performed_counters(),
            execution_stage_outcomes: Vec::new(),
            lineage_records: lineage_delta(lineage_before, lineage_after),
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

    fn settle_declared_mutation(
        &mut self,
    ) -> Result<std::collections::BTreeSet<LocalitySemanticOutputId>, SignalError> {
        self.settle_mutations(&[self.locality_definition().mutation()])
    }

    fn apply_declared_mutation(&mut self) -> Result<(), SignalError> {
        self.apply_mutations(&[self.locality_definition().mutation()])
    }

    fn baseline_retained_outputs(
        &self,
        evaluated: &std::collections::BTreeSet<LocalitySemanticOutputId>,
    ) -> Result<std::collections::BTreeSet<LocalitySemanticOutputId>, SignalError> {
        evaluated
            .iter()
            .filter_map(|output| {
                let declaration = &self.locality_definition().outputs()[output.ordinal() as usize];
                let node = self.handles[output];
                let retained = declaration.produced_aspects().iter().all(|aspect| {
                    self.runtime
                        .graph()
                        .node_version_for_scope(node, signal_aspect(*aspect), None)
                        .is_ok_and(|version| {
                            version
                                == self
                                    .locality_definition()
                                    .workload()
                                    .baseline_aspect_version()
                        })
                });
                retained.then_some(Ok(*output))
            })
            .collect()
    }

    pub(super) fn definition(&self) -> &FinancialWorldDefinition {
        &self.definition
    }

    fn locality_definition(&self) -> &FinancialLocalityDefinition {
        self.definition
            .locality()
            .expect("compiled locality backend retains locality definition")
    }

    fn committed_financial_values(
        &self,
    ) -> Result<BTreeMap<LocalitySemanticOutputId, i64>, SignalError> {
        self.handles
            .iter()
            .map(|(output, node)| self.committed_value(*output, *node))
            .collect()
    }

    fn committed_value(
        &self,
        output: LocalitySemanticOutputId,
        node: NodeId,
    ) -> Result<(LocalitySemanticOutputId, i64), SignalError> {
        let identity = self
            .runtime
            .graph()
            .node_runtime_artifact_warm(node)?
            .and_then(|warm| warm.output_identity.as_ref())
            .ok_or_else(|| {
                SignalError::internal(format!(
                    "locality output {output:?} lacks a committed artifact identity"
                ))
            })?;
        let (_, value) = identity.as_str().rsplit_once(':').ok_or_else(|| {
            SignalError::internal(format!(
                "locality output {output:?} has malformed financial identity"
            ))
        })?;
        let value = value.parse::<i64>().map_err(|_| {
            SignalError::internal(format!(
                "locality output {output:?} has non-numeric financial identity"
            ))
        })?;
        Ok((output, value))
    }
}

impl super::CompiledFinancialWorld {
    pub(super) fn from_locality(locality: CompiledFinancialLocalityWorld) -> Self {
        Self {
            kind: super::CompiledFinancialWorldKind::Locality(locality),
        }
    }

    pub(super) fn locality(&self) -> &CompiledFinancialLocalityWorld {
        match &self.kind {
            super::CompiledFinancialWorldKind::Locality(locality) => locality,
            super::CompiledFinancialWorldKind::Portfolio(_) => {
                panic!("locality operation used with compiled portfolio world")
            }
        }
    }

    pub(super) fn locality_mut(&mut self) -> &mut CompiledFinancialLocalityWorld {
        match &mut self.kind {
            super::CompiledFinancialWorldKind::Locality(locality) => locality,
            super::CompiledFinancialWorldKind::Portfolio(_) => {
                panic!("locality mutation used with compiled portfolio world")
            }
        }
    }
}
