use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::node::NodeState;
use crate::data::proof::invalidation::binding::DependencyRevision;
use crate::data::proof::invalidation::progression::{
    InvalidationStageOrder, ReadyInvalidationBatch,
};
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::facade::{mark_dirty, EvaluationRequestMode};
use crate::logic::context::EvaluationContext;
use crate::logic::invalidation::scheduling::{
    admit_current_readiness, execute_ready, lower_current_work,
};
use crate::logic::planner::{StageExecutionOutcome, StageExecutionRecord};
use crate::tests::domains::fintech::world::{
    FinancialLocalityAction, FinancialLocalitySubscription, LocalitySemanticOutputId,
};
use crate::tests::leased_execution::support::{authority, request};

use super::super::physical_ready::captured_bindings;
use super::super::PhysicalReadyWitness;
use super::super::{signal_aspect, CompiledFinancialLocalityWorld};
use super::churn_program::ChurnEvaluationProgram;
use super::LocalityExecutionSettlement;

pub(super) fn run_churn_trace(
    world: &mut CompiledFinancialLocalityWorld,
    trace_index: usize,
    workers: usize,
) -> Result<LocalityExecutionSettlement, SignalError> {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let actions = world.locality_definition().action_traces()[trace_index]
        .actions()
        .to_vec();
    let program = ChurnEvaluationProgram::new(
        world.locality_definition().outputs(),
        &world.handles,
        &world.baseline_values,
    );
    let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
    let mut staged = BTreeMap::<u16, ReadyInvalidationBatch>::new();
    let mut stage_outcomes = Vec::new();
    let mut stage_records = Vec::new();
    let mut physical_ready = PhysicalReadyWitness::default();
    for (index, action) in actions.iter().copied().enumerate() {
        match action {
            FinancialLocalityAction::CommitFactor(mutation) => {
                program.publish(mutation)?;
                let source = world.handles[&mutation.producer];
                mark_dirty(
                    world.runtime.graph_mut(),
                    source,
                    signal_aspect(mutation.aspect),
                )?;
                let before = captured_bindings(world.runtime.graph());
                world
                    .runtime
                    .transaction(request_execution, &mut (), |tx| {
                        tx.read(source, &evaluator).map(|_| ())
                    })?;
                physical_ready.record_transaction(
                    source,
                    world.runtime.graph(),
                    &before,
                    &captured_bindings(world.runtime.graph()),
                )?;
                let is_pre_rewire = actions.get(index + 1).is_some_and(|next| {
                    matches!(next, FinancialLocalityAction::StagePreRewireWork { .. })
                });
                if !is_pre_rewire {
                    settle_current_frontier(
                        world,
                        &program,
                        workers,
                        &mut stage_outcomes,
                        &mut stage_records,
                        &mut physical_ready,
                    )?;
                }
            }
            FinancialLocalityAction::StagePreRewireWork { round, binding } => {
                let target = world.handles[&binding.target];
                if world.runtime.graph().dependency_revision(target)?
                    != DependencyRevision(binding.dependency_revision)
                    || world.runtime.graph().get_state(target)? != NodeState::Dirty
                {
                    return Err(SignalError::internal(
                        "pre-rewire financial work was not current and causally dirty",
                    ));
                }
                let ready = current_ready(world, target)?;
                if staged.insert(round, ready).is_some() {
                    return Err(SignalError::internal("duplicate staged churn round"));
                }
            }
            FinancialLocalityAction::AcceptedOwnerMove { round, change } => {
                if !staged.contains_key(&round) {
                    return Err(SignalError::internal("owner move lacks staged prior work"));
                }
                program.accept_owner_move(change)?;
                set_subscription(world, change.target, Some(change.after_subscription))?;
                require_revision(
                    world,
                    change.target,
                    change.structural.resulting_dependency_revision,
                )?;
            }
            FinancialLocalityAction::RejectStaleWork { round, .. } => {
                let ready = staged
                    .remove(&round)
                    .ok_or_else(|| SignalError::internal("stale churn work was not staged"))?;
                let effects = AtomicUsize::new(0);
                let denial = execute_ready(world.runtime.graph(), ready, || {
                    effects.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .expect_err("pre-rewire ready work must be stale");
                if !denial.to_string().contains("stale dependency revision")
                    || effects.load(Ordering::SeqCst) != 0
                {
                    return Err(SignalError::internal(
                        "stale churn work crossed the effect boundary",
                    ));
                }
            }
            FinancialLocalityAction::AcceptedDependencyRemoval {
                removed_subscription,
                structural,
                ..
            } => {
                program.remove_subscription(structural.target, removed_subscription)?;
                set_subscription(world, structural.target, None)?;
                require_revision(
                    world,
                    structural.target,
                    structural.resulting_dependency_revision,
                )?;
            }
            FinancialLocalityAction::AcceptedDependencyRecreation {
                subscription,
                structural,
                ..
            } => {
                program.recreate_subscription(structural.target, subscription)?;
                set_subscription(world, structural.target, Some(subscription))?;
                require_revision(
                    world,
                    structural.target,
                    structural.resulting_dependency_revision,
                )?;
            }
            FinancialLocalityAction::RejectedCycle {
                target,
                attempted_subscription,
                retained_dependency_revision,
                ..
            } => reject_cycle_atomically(
                world,
                target,
                attempted_subscription,
                retained_dependency_revision,
            )?,
            _ => {
                return Err(SignalError::internal(
                    "portfolio churn trace contains a non-churn lifecycle action",
                ));
            }
        }
    }
    if !staged.is_empty() {
        return Err(SignalError::internal("churn trace left stale work staged"));
    }
    Ok(LocalityExecutionSettlement {
        evaluated_outputs: program.evaluated_outputs(),
        stage_outcomes,
        stage_records,
        physical_ready: Some(physical_ready),
    })
}

fn current_ready(
    world: &mut CompiledFinancialLocalityWorld,
    target: crate::data::handle::NodeId,
) -> Result<ReadyInvalidationBatch, SignalError> {
    let mut graph = world.runtime.graph_mut();
    let epoch = graph.begin_invalidation_readiness_epoch();
    let order = InvalidationStageOrder { stage: 0, order: 0 };
    let NodeInvalidationInput::Resolved(input) = graph.node_invalidation_input(target)? else {
        return Err(SignalError::internal(
            "staged churn target lacks resolved dependency authority",
        ));
    };
    let lowered = lower_current_work(&graph, target, input, epoch, order)?;
    admit_current_readiness(&graph, lowered, epoch, order)
}

fn settle_current_frontier(
    world: &mut CompiledFinancialLocalityWorld,
    program: &ChurnEvaluationProgram,
    workers: usize,
    stage_outcomes: &mut Vec<StageExecutionOutcome>,
    stage_records: &mut Vec<StageExecutionRecord>,
    physical_ready: &mut PhysicalReadyWitness,
) -> Result<(), SignalError> {
    let waves = world
        .locality_definition()
        .workload()
        .release_waves()
        .to_vec();
    for wave in waves {
        let nodes = wave
            .iter()
            .map(|output| world.handles[output])
            .filter(|node| {
                world
                    .runtime
                    .graph()
                    .get_state(*node)
                    .is_ok_and(|state| state != NodeState::Clean)
            })
            .collect::<Vec<_>>();
        if nodes.is_empty() {
            continue;
        }
        // Cover the complete churn wave under a finite request shared by
        // serial and parallel certification postures.
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
        let lease = authority()
            .request_lease(admission)
            .map_err(|_| SignalError::invalid_input("churn host lease denied"))?;
        let before = captured_bindings(world.runtime.graph());
        let report = world.runtime.evaluate_checked(
            &nodes,
            EvaluationRequestMode::Default,
            &(),
            &|view| program.evaluate_checked(view),
            &lease,
        )?;
        physical_ready.record_checked(
            &before,
            &captured_bindings(world.runtime.graph()),
            &report.stages,
        )?;
        program.record_completed(&report);
        for stage in report.stages {
            stage_outcomes.push(stage.outcome);
            stage_records.push(stage);
        }
    }
    Ok(())
}

fn set_subscription(
    world: &mut CompiledFinancialLocalityWorld,
    target: LocalitySemanticOutputId,
    subscription: Option<FinancialLocalitySubscription>,
) -> Result<(), SignalError> {
    let target = world.handles[&target];
    let edges = subscription
        .into_iter()
        .map(|subscription| dependency_edge(world, subscription))
        .collect::<Vec<_>>();
    world.runtime.graph_mut().set_dependencies(target, edges)
}

fn dependency_edge(
    world: &CompiledFinancialLocalityWorld,
    subscription: FinancialLocalitySubscription,
) -> DependencyEdge {
    let source = world.handles[&subscription.upstream];
    match subscription.edge_scope {
        None => DependencyEdge::new(source, signal_aspect(subscription.input_aspect)),
        Some(scope) => DependencyEdge::partition_detail(
            source,
            signal_aspect(subscription.input_aspect),
            scope.partition_label(),
            scope.detail_label().expect("churn detail scope"),
        ),
    }
}

fn require_revision(
    world: &CompiledFinancialLocalityWorld,
    target: LocalitySemanticOutputId,
    expected: u64,
) -> Result<(), SignalError> {
    if world
        .runtime
        .graph()
        .dependency_revision(world.handles[&target])?
        == DependencyRevision(expected)
    {
        Ok(())
    } else {
        Err(SignalError::internal(
            "accepted churn mutation produced the wrong dependency revision",
        ))
    }
}

fn reject_cycle_atomically(
    world: &mut CompiledFinancialLocalityWorld,
    target: LocalitySemanticOutputId,
    attempted: FinancialLocalitySubscription,
    retained_revision: u64,
) -> Result<(), SignalError> {
    let target_node = world.handles[&target];
    let retained_edges = world.runtime.graph().dependencies_of(target_node)?.to_vec();
    let attempted_edge = dependency_edge(world, attempted);
    if world
        .runtime
        .graph_mut()
        .set_dependencies(target_node, [attempted_edge])
        .is_ok()
    {
        return Err(SignalError::internal("declared churn cycle was accepted"));
    }
    if world.runtime.graph().dependencies_of(target_node)? != retained_edges
        || world.runtime.graph().dependency_revision(target_node)?
            != DependencyRevision(retained_revision)
    {
        return Err(SignalError::internal(
            "rejected churn cycle mutated live topology authority",
        ));
    }
    Ok(())
}
