//! Signal alone interprets graph settlement. The generic proof door sees only
//! carried binding axes and canonical opaque effect keys.

use super::eligibility::PrevalidatedTask;
use crate::data::aspect::MAX_ASPECTS;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::{EvaluationCondition, NodeState};
use crate::data::proof::invalidation::progression::{
    DisjointGraphBatch, GraphBatchBindingAxes, GraphMemberBinding, GraphProposalKey,
    InvalidationProgressionOwner,
};
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
use crate::logic::planner::EligibleTask;
use worth_execution::MapKernelContext;

mod resource_admission;
pub(in crate::logic::planner) use resource_admission::{
    CheckedApplyCapacity, CheckedEpochAdmissionGrant,
};

pub(crate) enum EpochAdmission<'a> {
    LegacySerial,
    Checked(CheckedEpochAdmission<'a>),
}

/// The accepted prefix and its capacity grant are one move-only value. Only
/// this module selects the range after the resource owner admits each member.
pub(crate) struct CheckedEpochAdmission<'a> {
    tasks: &'a [EligibleTask],
    task_offset: usize,
    grant: CheckedEpochAdmissionGrant,
}

impl CheckedEpochAdmission<'_> {
    pub(crate) fn tasks(&self) -> &[EligibleTask] {
        self.tasks
    }

    pub(crate) fn task_offset(&self) -> usize {
        self.task_offset
    }

    pub(crate) fn result_grant_bytes(&self) -> u64 {
        self.grant.result_grant_bytes()
    }

    pub(in crate::logic::planner) fn apply_capacity(&self) -> &CheckedApplyCapacity {
        self.grant.apply_capacity()
    }
}

impl<'a> CheckedEpochAdmission<'a> {
    pub(in crate::logic::planner) fn into_parts(
        self,
    ) -> (&'a [EligibleTask], CheckedApplyCapacity) {
        let (_, _, apply) = self.grant.into_parts();
        (self.tasks, apply)
    }
}

/// Open discovery is an ordering barrier. A later epoch is considered only
/// after this prefix has reconciled and published through the owner.
pub(crate) fn epoch_width<'a>(
    graph: &mut SignalGraph,
    tasks: &'a [EligibleTask],
    task_offset: usize,
    checked: bool,
    request: worth_execution::ExecutionRequest<'_, '_>,
    comparator: &impl ComparatorPolicyResolver,
    mut preparation: Option<&mut SignalPreparationBudget>,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
) -> Result<EpochAdmission<'a>, SignalError> {
    if tasks.is_empty() {
        return Err(SignalError::invalid_input(
            "an epoch requires a nonempty task prefix",
        ));
    }
    super::work::checkpoint(work.as_deref_mut(), 1)?;
    if !checked {
        if request.is_leased() {
            return Err(SignalError::invalid_input(
                "a leased Signal request requires a checked evaluator",
            ));
        }
        return Ok(EpochAdmission::LegacySerial);
    }
    let can_group = bounded_eligible(graph, &tasks[0])?;
    graph.prepare_epoch_topology_storage_readiness(
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let budget = preparation
        .ok_or_else(|| SignalError::invalid_input("checked epoch requires request preparation"))?;
    let mut resources = resource_admission::ResourceAdmission::new(budget, request)?;
    if !resources.consider(graph, &tasks[0], comparator, work.as_deref_mut(), budget)? {
        return Err(SignalError::internal(
            "singleton epoch admission declined its task",
        ));
    }
    if !can_group {
        return Ok(EpochAdmission::Checked(CheckedEpochAdmission {
            tasks: &tasks[..1],
            task_offset,
            grant: resources.finish()?,
        }));
    }
    let mut width = 1;
    for next in &tasks[1..] {
        super::work::checkpoint(work.as_deref_mut(), 1)?;
        if !bounded_eligible(graph, next)? {
            break;
        }
        let mut compatible = true;
        for prior in &tasks[..width] {
            let left = graph.get_contract(prior.node)?;
            let right = graph.get_contract(next.node)?;
            let left_inputs = left
                .execution
                .bounded_inputs
                .as_ref()
                .expect("checked above");
            let right_inputs = right
                .execution
                .bounded_inputs
                .as_ref()
                .expect("checked above");
            super::work::checkpoint(
                work.as_deref_mut(),
                left_inputs
                    .copy_work_bound()
                    .saturating_add(right_inputs.copy_work_bound())
                    .saturating_add(1),
            )?;
            if prior.node == next.node
                || left_inputs
                    .as_slice()
                    .iter()
                    .any(|input| input.source == next.node)
                || right_inputs
                    .as_slice()
                    .iter()
                    .any(|input| input.source == prior.node)
            {
                compatible = false;
                break;
            }
        }
        if !compatible {
            break;
        }
        if !resources.consider(graph, next, comparator, work.as_deref_mut(), budget)? {
            break;
        }
        width += 1;
    }
    let grant = resources.finish()?;
    debug_assert_eq!(grant.width().get(), width);
    Ok(EpochAdmission::Checked(CheckedEpochAdmission {
        tasks: &tasks[..width],
        task_offset,
        grant,
    }))
}

fn bounded_eligible(graph: &SignalGraph, task: &EligibleTask) -> Result<bool, SignalError> {
    let contract = graph.get_contract(task.node)?;
    Ok(contract.execution.bounded_inputs.is_some()
        && matches!(
            graph.node_eval_config(task.node)?.condition,
            EvaluationCondition::Always
                | EvaluationCondition::AspectFilter(_)
                | EvaluationCondition::DeltaThreshold(_)
        )
        && !matches!(
            task.request_mode,
            crate::logic::evaluation::EvaluationRequestMode::ForceOnDemand
        ))
}

pub(super) fn admit(
    graph: &SignalGraph,
    tasks: &[EligibleTask],
    task_offset: usize,
    validated: &[PrevalidatedTask],
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<DisjointGraphBatch, SignalError> {
    let expected = current_binding(
        graph,
        tasks,
        task_offset,
        validated,
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    preparation_budget::claim_vec::<(crate::data::handle::NodeId, Vec<GraphProposalKey>)>(
        preparation.as_deref_mut(),
        tasks.len(),
    )?;
    for _ in tasks {
        super::work::checkpoint(work.as_deref_mut(), MAX_ASPECTS + 8)?;
        preparation_budget::claim_vec::<GraphProposalKey>(
            preparation.as_deref_mut(),
            MAX_ASPECTS + 7,
        )?;
    }
    // Batch identity is canonical node order, independently of worker count.
    let current = current_binding(graph, tasks, task_offset, validated, work, preparation)?;
    InvalidationProgressionOwner::admit_graph_batch(expected, current)
}

pub(super) fn current_binding(
    graph: &SignalGraph,
    tasks: &[EligibleTask],
    task_offset: usize,
    validated: &[PrevalidatedTask],
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<GraphBatchBindingAxes, SignalError> {
    super::work::checkpoint(work.as_deref_mut(), tasks.len())?;
    preparation_budget::claim_vec::<GraphMemberBinding>(preparation.as_deref_mut(), tasks.len())?;
    let mut members = Vec::with_capacity(tasks.len());
    for (index, (task, validation)) in tasks.iter().zip(validated).enumerate() {
        let declaration = graph
            .get_contract(task.node)?
            .execution
            .bounded_inputs
            .as_ref()
            .ok_or_else(|| {
                SignalError::invalid_input("graph batch needs bounded input declarations")
            })?;
        super::work::checkpoint(
            work.as_deref_mut(),
            declaration.copy_work_bound().saturating_add(1),
        )?;
        preparation_budget::claim_vec::<u64>(
            preparation.as_deref_mut(),
            declaration.as_slice().len(),
        )?;
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<crate::data::node::DeclaredSignalInput>(
                declaration.as_slice().len(),
            )?;
            budget.claim(declaration.captured_scope_heap_bound().ok_or_else(|| {
                SignalError::invalid_input("graph declaration clone memory overflow")
            })?)?;
        }
        let mut versions = Vec::with_capacity(declaration.as_slice().len());
        for input in declaration.as_slice() {
            if graph.get_state(input.source)? != NodeState::Clean {
                return Err(SignalError::invalid_input(
                    "declared producer is not settled",
                ));
            }
            versions.push(graph.node_version_for_scope(
                input.source,
                input.aspect,
                input.scope.as_ref(),
            )?);
        }
        let invalidation = match validation {
            PrevalidatedTask::NeedsCompute {
                ready_invalidation: Some(ready),
                ..
            } => {
                crate::logic::invalidation::scheduling::ensure_ready_is_current(graph, ready)?;
                if let crate::data::proof::invalidation::progression::InvalidationOriginBinding::DependencyCommit {
                    producer_commit_ordinals, ..
                } = &InvalidationProgressionOwner::ready_binding(ready).origin {
                    preparation_budget::claim_vec::<crate::data::proof::invalidation::binding::OutputCommitOrdinal>(
                        preparation.as_deref_mut(), producer_commit_ordinals.len())?;
                }
                Some(InvalidationProgressionOwner::ready_binding(ready).clone())
            }
            _ => None,
        };
        members.push(GraphMemberBinding {
            target: task.node,
            task_order: task_offset + index,
            declaration: declaration.clone(),
            producer_versions: versions,
            dependency_revision: graph.node_dependency_revision(task.node)?,
            invalidation,
            produced_aspects: graph.get_contract(task.node)?.semantics.produces,
        });
    }
    members.sort_unstable_by_key(|member| member.target);
    Ok(GraphBatchBindingAxes {
        graph_instance: graph.installed_graph_capability().graph_instance_id(),
        epoch: graph.current_invalidation_readiness_epoch(),
        observation_generation: graph.observation_session_active_generation(),
        members,
    })
}
