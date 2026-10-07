use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::output::{NodeEvaluationResult, PartitionSubscription};
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationOutput;
use crate::tests::domains::fintech::world::{
    FinancialAspect, FinancialLocalityFormula, FinancialLocalityMutation, FinancialLocalityOutput,
    FinancialLocalitySubscription, FinancialLocalityTopologyChange, LocalitySemanticOutputId,
};

use super::super::signal_aspect;

#[derive(Clone)]
pub(super) struct ChurnEvaluationProgram {
    handles: BTreeMap<LocalitySemanticOutputId, NodeId>,
    outputs_by_node: BTreeMap<NodeId, LocalitySemanticOutputId>,
    state: Arc<Mutex<ChurnProgramState>>,
}

struct ChurnProgramState {
    outputs: Vec<FinancialLocalityOutput>,
    current_values: BTreeMap<LocalitySemanticOutputId, i64>,
    committed_values: BTreeMap<LocalitySemanticOutputId, i64>,
    versions: BTreeMap<(LocalitySemanticOutputId, FinancialAspect), u64>,
    evaluated: BTreeSet<LocalitySemanticOutputId>,
}

impl ChurnEvaluationProgram {
    pub(super) fn new(
        outputs: &[FinancialLocalityOutput],
        handles: &BTreeMap<LocalitySemanticOutputId, NodeId>,
        baseline_values: &BTreeMap<LocalitySemanticOutputId, i64>,
    ) -> Self {
        Self {
            handles: handles.clone(),
            outputs_by_node: handles.iter().map(|(id, node)| (*node, *id)).collect(),
            state: Arc::new(Mutex::new(ChurnProgramState {
                outputs: outputs.to_vec(),
                current_values: baseline_values.clone(),
                committed_values: baseline_values.clone(),
                versions: outputs
                    .iter()
                    .flat_map(|output| {
                        output
                            .produced_aspects()
                            .into_iter()
                            .map(move |aspect| ((output.id, aspect), 1))
                    })
                    .collect(),
                evaluated: BTreeSet::new(),
            })),
        }
    }

    pub(super) fn publish(&self, mutation: FinancialLocalityMutation) -> Result<(), SignalError> {
        let mut state = self.state.lock().expect("churn program state poisoned");
        let output = &state.outputs[mutation.producer.ordinal() as usize];
        let FinancialLocalityFormula::MarketSource { mutation_delta, .. } = output.formula else {
            return Err(SignalError::internal(
                "churn publication target is not a market source",
            ));
        };
        state
            .current_values
            .entry(mutation.producer)
            .and_modify(|value| {
                *value = value
                    .checked_add(mutation_delta)
                    .expect("churn market value overflow")
            });
        state.recompute_dependencies()
    }

    pub(super) fn accept_owner_move(
        &self,
        change: FinancialLocalityTopologyChange,
    ) -> Result<(), SignalError> {
        let mut state = self.state.lock().expect("churn program state poisoned");
        let output = &mut state.outputs[change.target.ordinal() as usize];
        if output.owner != change.before_owner
            || output.subscriptions != [change.before_subscription]
        {
            return Err(SignalError::internal(
                "churn owner move did not match current financial topology",
            ));
        }
        output.owner = change.after_owner;
        output.subscriptions = vec![change.after_subscription];
        state.recompute_dependencies()
    }

    pub(super) fn remove_subscription(
        &self,
        target: LocalitySemanticOutputId,
        removed: FinancialLocalitySubscription,
    ) -> Result<(), SignalError> {
        let mut state = self.state.lock().expect("churn program state poisoned");
        let output = &mut state.outputs[target.ordinal() as usize];
        if output.subscriptions != [removed] {
            return Err(SignalError::internal(
                "churn removal did not match current financial topology",
            ));
        }
        output.subscriptions.clear();
        state.recompute_dependencies()
    }

    pub(super) fn recreate_subscription(
        &self,
        target: LocalitySemanticOutputId,
        subscription: FinancialLocalitySubscription,
    ) -> Result<(), SignalError> {
        let mut state = self.state.lock().expect("churn program state poisoned");
        let output = &mut state.outputs[target.ordinal() as usize];
        if !output.subscriptions.is_empty() {
            return Err(SignalError::internal(
                "churn recreation requires an absent dependency",
            ));
        }
        output.subscriptions.push(subscription);
        state.recompute_dependencies()
    }

    pub(super) fn evaluate(
        &self,
        view: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        let output_id = self.outputs_by_node[&view.node()];
        let output = {
            let state = self.state.lock().expect("churn program state poisoned");
            state.outputs[output_id.ordinal() as usize].clone()
        };
        for subscription in &output.subscriptions {
            let source = self.handles[&subscription.upstream];
            match subscription.edge_scope {
                None => {
                    view.read_aspect_version(source, signal_aspect(subscription.input_aspect))?;
                }
                Some(scope) => {
                    view.read_partitioned_aspect_version(
                        source,
                        signal_aspect(subscription.input_aspect),
                        PartitionSubscription::partition_and_detail(
                            scope.partition_label(),
                            scope.detail_label().expect("churn detail scope"),
                        ),
                    )?;
                }
            }
        }
        let result = self.result_for(&output);
        Ok(view.finish(result))
    }

    pub(super) fn evaluate_checked(
        &self,
        view: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        view.work()
            .checkpoint(self.outputs_by_node.len().saturating_add(256) as u64)
            .map_err(|_| SignalError::invalid_input("churn kernel work exhausted"))?;
        let output_id = self.outputs_by_node[&view.node()];
        let state = self.state.lock().expect("churn program state poisoned");
        let output = &state.outputs[output_id.ordinal() as usize];
        for subscription in &output.subscriptions {
            let source = self.handles[&subscription.upstream];
            let aspect = signal_aspect(subscription.input_aspect);
            match subscription.edge_scope {
                None => {
                    view.read(source, aspect)?;
                }
                Some(scope) => {
                    let scope = PartitionSubscription::partition_and_detail(
                        scope.partition_label(),
                        scope.detail_label().expect("churn detail scope"),
                    );
                    view.read_scoped(source, aspect, &scope)?;
                }
            }
        }
        Ok(view.finish(state.preview_result(output)))
    }

    pub(super) fn record_completed(&self, report: &crate::logic::planner::ExecutionReport) {
        let mut state = self.state.lock().expect("churn program state poisoned");
        for stage in &report.stages {
            for task in &stage.task_records {
                if let Some(output) = self
                    .outputs_by_node
                    .get(&task.node)
                    .filter(|_| task.recomputed)
                {
                    state.commit_output(*output);
                }
            }
        }
    }

    pub(super) fn evaluated_outputs(&self) -> BTreeSet<LocalitySemanticOutputId> {
        self.state
            .lock()
            .expect("churn program state poisoned")
            .evaluated
            .clone()
    }

    fn result_for(&self, output: &FinancialLocalityOutput) -> NodeEvaluationResult {
        let mut state = self.state.lock().expect("churn program state poisoned");
        state.commit_output(output.id);
        state.preview_result(output)
    }
}

impl ChurnProgramState {
    fn commit_output(&mut self, output_id: LocalitySemanticOutputId) {
        let value = self.current_values[&output_id];
        let changed = self.committed_values[&output_id] != value;
        if changed {
            self.committed_values.insert(output_id, value);
            for aspect in self.outputs[output_id.ordinal() as usize].produced_aspects() {
                *self.versions.get_mut(&(output_id, aspect)).unwrap() += 1;
            }
        }
        self.evaluated.insert(output_id);
    }

    fn preview_result(&self, output: &FinancialLocalityOutput) -> NodeEvaluationResult {
        let value = self.current_values[&output.id];
        let changed = self.committed_values[&output.id] != value;
        let version =
            output
                .produced_aspects()
                .into_iter()
                .fold(AspectVersion::zero(), |version, aspect| {
                    version.with(
                        signal_aspect(aspect),
                        self.versions[&(output.id, aspect)] + u64::from(changed),
                    )
                });
        NodeEvaluationResult::from_version(version).with_output_identity(format!(
            "financial-locality:{:?}:{:?}:{}:{}",
            output.owner,
            output.role,
            output.id.ordinal(),
            value
        ))
    }
    fn recompute_dependencies(&mut self) -> Result<(), SignalError> {
        for output in &self.outputs {
            let value = match output.formula {
                FinancialLocalityFormula::MarketSource { .. } => continue,
                FinancialLocalityFormula::StableControl { retained_value } => retained_value,
                FinancialLocalityFormula::LinearDependency {
                    multiplier_micros,
                    basis_value,
                } => output
                    .subscriptions
                    .iter()
                    .map(|subscription| self.current_values[&subscription.upstream])
                    .sum::<i64>()
                    .checked_mul(multiplier_micros)
                    .and_then(|value| value.checked_div(1_000_000))
                    .and_then(|value| value.checked_add(basis_value))
                    .ok_or_else(|| SignalError::internal("churn dependency value overflow"))?,
            };
            self.current_values.insert(output.id, value);
        }
        Ok(())
    }
}
