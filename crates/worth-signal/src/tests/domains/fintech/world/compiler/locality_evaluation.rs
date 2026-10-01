use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::output::{ChangedRegion, NodeEvaluationResult};
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationOutput;

use super::super::{
    FinancialLocalityDefinition, FinancialLocalityFormula, FinancialLocalityMutation,
    FinancialLocalityOutput, LocalitySemanticOutputId,
};
use super::locality_topology::partition_subscription;
use super::topology::signal_aspect;

pub(super) struct LocalityEvaluationProgram {
    outputs_by_node: BTreeMap<NodeId, FinancialLocalityOutput>,
    handles: BTreeMap<LocalitySemanticOutputId, NodeId>,
    financial_values: BTreeMap<LocalitySemanticOutputId, i64>,
    baseline_financial_values: BTreeMap<LocalitySemanticOutputId, i64>,
    aspect_version: u64,
    baseline_aspect_version: u64,
    mutations: Vec<FinancialLocalityMutation>,
    evaluated_outputs: Arc<Mutex<BTreeSet<LocalitySemanticOutputId>>>,
}

struct LocalityValueState<'a> {
    current: &'a BTreeMap<LocalitySemanticOutputId, i64>,
    baseline: &'a BTreeMap<LocalitySemanticOutputId, i64>,
    aspect_version: u64,
}

impl LocalityEvaluationProgram {
    pub(super) fn baseline(
        definition: &FinancialLocalityDefinition,
        handles: &BTreeMap<LocalitySemanticOutputId, NodeId>,
        baseline_values: &BTreeMap<LocalitySemanticOutputId, i64>,
    ) -> Self {
        Self::with_values(
            definition,
            handles,
            LocalityValueState {
                current: baseline_values,
                baseline: baseline_values,
                aspect_version: definition.workload().baseline_aspect_version(),
            },
            &[],
        )
    }

    pub(super) fn shocked(
        definition: &FinancialLocalityDefinition,
        handles: &BTreeMap<LocalitySemanticOutputId, NodeId>,
        baseline_values: &BTreeMap<LocalitySemanticOutputId, i64>,
        shocked_values: &BTreeMap<LocalitySemanticOutputId, i64>,
        mutations: &[FinancialLocalityMutation],
    ) -> Self {
        Self::with_values(
            definition,
            handles,
            LocalityValueState {
                current: shocked_values,
                baseline: baseline_values,
                aspect_version: definition.workload().mutation_aspect_version(),
            },
            mutations,
        )
    }

    pub(super) fn shocked_for_batch(
        definition: &FinancialLocalityDefinition,
        handles: &BTreeMap<LocalitySemanticOutputId, NodeId>,
        baseline_financial_values: &BTreeMap<LocalitySemanticOutputId, i64>,
        shocked_values: &BTreeMap<LocalitySemanticOutputId, i64>,
        mutations: &[FinancialLocalityMutation],
        batch_index: usize,
    ) -> Self {
        Self::with_values(
            definition,
            handles,
            LocalityValueState {
                current: shocked_values,
                baseline: baseline_financial_values,
                aspect_version: definition
                    .workload()
                    .mutation_aspect_version()
                    .saturating_add(batch_index as u64),
            },
            mutations,
        )
    }

    fn with_values(
        definition: &FinancialLocalityDefinition,
        handles: &BTreeMap<LocalitySemanticOutputId, NodeId>,
        values: LocalityValueState<'_>,
        mutations: &[FinancialLocalityMutation],
    ) -> Self {
        Self {
            outputs_by_node: definition
                .outputs()
                .iter()
                .map(|output| (handles[&output.id], output.clone()))
                .collect(),
            handles: handles.clone(),
            financial_values: values.current.clone(),
            baseline_financial_values: values.baseline.clone(),
            aspect_version: values.aspect_version,
            baseline_aspect_version: definition.workload().baseline_aspect_version(),
            mutations: mutations.to_vec(),
            evaluated_outputs: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub(super) fn evaluate(
        &self,
        view: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        let output = self.outputs_by_node.get(&view.node()).ok_or_else(|| {
            SignalError::invalid_input("locality evaluator received an unknown node")
        })?;
        self.evaluated_outputs
            .lock()
            .expect("locality evaluation identity lock poisoned")
            .insert(output.id);
        for subscription in &output.subscriptions {
            let source = self.handles[&subscription.upstream];
            match subscription.edge_scope {
                None => {
                    let _ =
                        view.read_aspect_version(source, signal_aspect(subscription.input_aspect))?;
                }
                Some(scope) => {
                    let _ = view.read_partitioned_aspect_version(
                        source,
                        signal_aspect(subscription.input_aspect),
                        partition_subscription(scope),
                    )?;
                }
            }
        }
        Ok(view.finish(self.result_for(output)))
    }

    pub(super) fn evaluate_checked(
        &self,
        view: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        view.work()
            .checkpoint(
                self.outputs_by_node
                    .len()
                    .saturating_add(self.mutations.len())
                    .saturating_add(256) as u64,
            )
            .map_err(|_| SignalError::invalid_input("locality kernel work exhausted"))?;
        let output = self.outputs_by_node.get(&view.node()).ok_or_else(|| {
            SignalError::invalid_input("locality evaluator received an unknown node")
        })?;
        for subscription in &output.subscriptions {
            let source = self.handles[&subscription.upstream];
            let aspect = signal_aspect(subscription.input_aspect);
            match subscription.edge_scope {
                None => {
                    view.read(source, aspect)?;
                }
                Some(scope) => {
                    view.read_scoped(source, aspect, &partition_subscription(scope))?;
                }
            }
        }
        Ok(view.finish(self.result_for(output)))
    }

    pub(super) fn record_completed(&self, report: &crate::logic::planner::ExecutionReport) {
        let mut evaluated = self
            .evaluated_outputs
            .lock()
            .expect("locality evaluation identity lock poisoned");
        for stage in &report.stages {
            for task in &stage.task_records {
                if let Some(output) = self
                    .outputs_by_node
                    .get(&task.node)
                    .filter(|_| task.recomputed)
                {
                    evaluated.insert(output.id);
                }
            }
        }
    }

    fn result_for(&self, output: &FinancialLocalityOutput) -> NodeEvaluationResult {
        let financial_value_changed =
            self.financial_values[&output.id] != self.baseline_financial_values[&output.id];
        let version =
            output
                .produced_aspects()
                .iter()
                .fold(AspectVersion::zero(), |version, aspect| {
                    let aspect_version = if !financial_value_changed
                        || (self
                            .mutations
                            .iter()
                            .any(|mutation| output.id == mutation.producer)
                            && !self.mutations.iter().any(|mutation| {
                                output.id == mutation.producer && *aspect == mutation.aspect
                            })) {
                        self.baseline_aspect_version
                    } else {
                        self.aspect_version
                    };
                    version.with(signal_aspect(*aspect), aspect_version)
                });
        let mut result = NodeEvaluationResult::from_version(version).with_output_identity(format!(
            "financial-locality:{:?}:{:?}:{}:{}",
            output.owner,
            output.role,
            output.id.ordinal(),
            self.financial_values[&output.id]
        ));
        if financial_value_changed {
            for mutation in self
                .mutations
                .iter()
                .filter(|mutation| mutation.producer == output.id)
            {
                if let Some(scope) = mutation.scope {
                    let mut region = ChangedRegion::new(scope.partition_label());
                    if let Some(detail) = scope.detail_label() {
                        region = region.with_detail(detail);
                    }
                    result =
                        result.with_changed_aspect_region(signal_aspect(mutation.aspect), region);
                }
            }
        }
        result
    }

    pub(super) fn evaluated_outputs(&self) -> BTreeSet<LocalitySemanticOutputId> {
        self.evaluated_outputs
            .lock()
            .expect("locality evaluation identity lock poisoned")
            .clone()
    }
}

pub(super) fn runtime_baseline_values(
    definition: &FinancialLocalityDefinition,
) -> Result<BTreeMap<LocalitySemanticOutputId, i64>, SignalError> {
    let mut values = BTreeMap::new();
    for output in definition.outputs() {
        let value = match output.formula {
            FinancialLocalityFormula::MarketSource { baseline_value, .. } => baseline_value,
            FinancialLocalityFormula::StableControl { retained_value } => retained_value,
            FinancialLocalityFormula::LinearDependency {
                multiplier_micros,
                basis_value,
            } => linear_value(output, &values, multiplier_micros, basis_value)?,
        };
        values.insert(output.id, value);
    }
    Ok(values)
}

pub(super) fn runtime_shocked_values(
    definition: &FinancialLocalityDefinition,
    baseline: &BTreeMap<LocalitySemanticOutputId, i64>,
    mutations: &[FinancialLocalityMutation],
) -> Result<BTreeMap<LocalitySemanticOutputId, i64>, SignalError> {
    runtime_shocked_values_for_batch(definition, baseline, mutations, 0)
}

pub(super) fn runtime_shocked_values_for_batch(
    definition: &FinancialLocalityDefinition,
    baseline: &BTreeMap<LocalitySemanticOutputId, i64>,
    mutations: &[FinancialLocalityMutation],
    batch_index: usize,
) -> Result<BTreeMap<LocalitySemanticOutputId, i64>, SignalError> {
    let shock_multiplier = i64::try_from(batch_index.saturating_add(1))
        .map_err(|_| SignalError::invalid_input("performance batch index exceeds i64"))?;
    let mut shocked = BTreeMap::new();
    for output in definition.outputs() {
        let value = match output.formula {
            FinancialLocalityFormula::MarketSource {
                baseline_value,
                mutation_delta,
                ..
            } if mutations
                .iter()
                .any(|mutation| output.id == mutation.producer) =>
            {
                baseline_value
                    .checked_add(
                        mutation_delta
                            .checked_mul(shock_multiplier)
                            .ok_or_else(|| SignalError::internal("locality shock overflow"))?,
                    )
                    .ok_or_else(|| SignalError::internal("locality source shock overflow"))?
            }
            FinancialLocalityFormula::MarketSource { baseline_value, .. } => baseline_value,
            FinancialLocalityFormula::StableControl { retained_value } => retained_value,
            FinancialLocalityFormula::LinearDependency {
                multiplier_micros,
                basis_value,
            } => shocked_linear_value(
                output,
                ShockEvaluationContext {
                    baseline,
                    shocked: &shocked,
                    mutations,
                },
                multiplier_micros,
                basis_value,
            )?,
        };
        shocked.insert(output.id, value);
    }
    Ok(shocked)
}

struct ShockEvaluationContext<'a> {
    baseline: &'a BTreeMap<LocalitySemanticOutputId, i64>,
    shocked: &'a BTreeMap<LocalitySemanticOutputId, i64>,
    mutations: &'a [super::super::FinancialLocalityMutation],
}

fn shocked_linear_value(
    output: &FinancialLocalityOutput,
    context: ShockEvaluationContext<'_>,
    multiplier_micros: i64,
    basis_value: i64,
) -> Result<i64, SignalError> {
    let inputs = output
        .subscriptions
        .iter()
        .map(|subscription| {
            let values = if context
                .mutations
                .iter()
                .any(|mutation| subscription.upstream == mutation.producer)
                && !context.mutations.iter().any(|mutation| {
                    subscription.upstream == mutation.producer
                        && subscription.input_aspect == mutation.aspect
                        && scopes_overlap(subscription.edge_scope, mutation.scope)
                }) {
                context.baseline
            } else {
                context.shocked
            };
            values.get(&subscription.upstream).copied().ok_or_else(|| {
                SignalError::internal("locality shock dependency is not topological")
            })
        })
        .sum::<Result<i64, SignalError>>()?;
    inputs
        .checked_mul(multiplier_micros)
        .and_then(|value| value.checked_div(1_000_000))
        .and_then(|value| value.checked_add(basis_value))
        .ok_or_else(|| SignalError::internal("locality shocked financial formula overflow"))
}

fn scopes_overlap(
    left: Option<super::super::LocalityScope>,
    right: Option<super::super::LocalityScope>,
) -> bool {
    match (left, right) {
        (None, _) | (_, None) => true,
        (Some(left), Some(right)) if left.region != right.region => false,
        (Some(left), Some(right)) => {
            left.detail.is_none() || right.detail.is_none() || left.detail == right.detail
        }
    }
}

fn linear_value(
    output: &FinancialLocalityOutput,
    values: &BTreeMap<LocalitySemanticOutputId, i64>,
    multiplier_micros: i64,
    basis_value: i64,
) -> Result<i64, SignalError> {
    let inputs = output
        .subscriptions
        .iter()
        .map(|subscription| {
            values.get(&subscription.upstream).copied().ok_or_else(|| {
                SignalError::internal("locality formula dependency is not topological")
            })
        })
        .sum::<Result<i64, SignalError>>()?;
    inputs
        .checked_mul(multiplier_micros)
        .and_then(|value| value.checked_div(1_000_000))
        .and_then(|value| value.checked_add(basis_value))
        .ok_or_else(|| SignalError::internal("locality financial formula overflow"))
}
