use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::proof::invalidation::progression::{
    InvalidationStageOrder, ReadyInvalidationBatch,
};
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::diagnostics::ReplayEventKind;
use crate::facade::{EvaluationRequestMode, NodeState};
use crate::logic::context::EvaluationContext;
use crate::logic::invalidation::scheduling::{
    admit_current_readiness, execute_ready, lower_current_work,
};
use crate::tests::domains::fintech::certification::invalidation::FreshFinancialLocalityRecompute;

use super::super::{signal_aspect, CompiledFinancialLocalityWorld, LocalityEvaluationProgram};

#[derive(Debug)]
pub(in crate::tests::domains::fintech) struct FinancialRestoreLifecycleEvidence {
    _private: (),
}

pub(super) fn certify_restore_lifecycle(
    world: &mut CompiledFinancialLocalityWorld,
) -> Result<FinancialRestoreLifecycleEvidence, SignalError> {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mutation = world.locality_definition().mutation();
    let main = world.runtime.current_branch();
    world.runtime.capture_snapshot()?;
    let analysis = world.runtime.create_branch("m13-locality-analysis")?;
    world.runtime.switch_branch(analysis.clone())?;
    publish_source(world)?;
    let target = world
        .locality_definition()
        .outputs()
        .iter()
        .find(|output| !output.subscriptions.is_empty())
        .map(|output| world.handles[&output.id])
        .ok_or_else(|| SignalError::internal("restore world lacks a dependency target"))?;
    let pre_restore_ready = current_ready(&mut world.runtime.graph_mut(), target)?;
    let source = world.handles[&mutation.producer];
    world
        .runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_changed(source, signal_aspect(mutation.aspect))
        })?;
    let source_basis = world
        .runtime
        .graph()
        .get_entry(world.handles[&mutation.producer])?
        .direct_invalidation_basis()
        .cloned()
        .ok_or_else(|| SignalError::internal("restore source basis was not staged"))?;
    let causes_before = world.runtime.graph().pending_causes(target)?.to_vec();
    if causes_before.is_empty() {
        return Err(SignalError::internal(
            "restore checkpoint lacks unresolved dependency causes",
        ));
    }
    let analysis_snapshot = world.runtime.capture_snapshot()?;
    let mut rerun = analysis_snapshot.authority_graph()?;
    world.runtime.switch_branch(main)?;
    world
        .runtime
        .restore_branch_snapshot(analysis.clone(), &analysis_snapshot)?;
    world.runtime.switch_branch(analysis.clone())?;
    let restore_replay = world.runtime.replay_for_branch(analysis.id);
    if world.runtime.graph().captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::ReplayDetail,
    ) && restore_replay
        .frames
        .iter()
        .all(|frame| frame.kind != ReplayEventKind::SnapshotRestored)
    {
        return Err(SignalError::internal(
            "branch restore did not publish its canonical replay event",
        ));
    }
    let restored_instance = world.runtime.graph().runtime_instance_id();
    let rerun_instance = rerun.runtime_instance_id();
    if restored_instance == rerun_instance {
        return Err(SignalError::internal(format!(
            "restore did not mint distinct reconstructed runtime authority: restored={restored_instance}, rerun={rerun_instance}",
        )));
    }
    let denial = execute_ready(world.runtime.graph(), pre_restore_ready, || Ok(()))
        .expect_err("pre-restore ready work must not execute in a fresh runtime");
    if !denial.to_string().contains("stale graph instance") {
        return Err(SignalError::internal(
            "pre-restore ready work failed for the wrong authority axis",
        ));
    }
    if world
        .runtime
        .graph()
        .get_entry(world.handles[&mutation.producer])?
        .direct_invalidation_basis()
        != Some(&source_basis)
    {
        return Err(SignalError::internal(
            "restore substituted the authoritative direct source basis",
        ));
    }
    require_semantic_causes(
        &causes_before,
        world.runtime.graph().pending_causes(target)?,
    )?;

    // Admission proves that the restored cause can form a current readiness
    // token.  Leave that token unconsumed so the subsequent production plan
    // settles the complete dependency wave and its downstream outputs.
    let _rebuilt = current_ready(&mut world.runtime.graph_mut(), target)?;
    let fresh = FreshFinancialLocalityRecompute::run_for_trace(
        world.locality_definition(),
        &world.locality_definition().action_traces()[0],
    );
    let shocked = fresh.shocked_values().clone();
    let program = LocalityEvaluationProgram::shocked(
        world.locality_definition(),
        &world.handles,
        &world.baseline_values,
        &shocked,
        &world.locality_definition().action_traces()[0].committed_mutations(),
    );
    let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
    let restored_targets = unsettled_nodes(world, world.runtime.graph())?;
    let rerun_targets = unsettled_nodes(world, &rerun)?;
    if restored_targets != rerun_targets {
        return Err(SignalError::internal(
            "restored locality runtimes disagree on unsettled work",
        ));
    }
    let plan = world
        .runtime
        .graph_mut()
        .build_evaluation_plan(&restored_targets, EvaluationRequestMode::Default)?;
    let rerun_plan = rerun.build_evaluation_plan(&rerun_targets, EvaluationRequestMode::Default)?;
    if canonical_plan_order(&plan) != canonical_plan_order(&rerun_plan) {
        return Err(SignalError::internal(
            "restored locality work order is not deterministic",
        ));
    }
    world
        .runtime
        .execute_prepared_plan(&plan, &(), &evaluator)?;
    rerun.execute_prepared_plan(&rerun_plan, &(), &evaluator)?;
    // A single plan can expose a newly dirty downstream wave only after its
    // upstream producer commits.  Continue with fresh plans until the
    // restored and independently reconstructed graphs are both settled; this
    // keeps the lifecycle oracle honest for deep partitioned chains.
    for _ in 0..=world.locality_definition().outputs().len() {
        let remaining = unsettled_nodes(world, world.runtime.graph())?;
        if remaining.is_empty() {
            break;
        }
        let next_plan = world
            .runtime
            .graph_mut()
            .build_evaluation_plan(&remaining, EvaluationRequestMode::Default)?;
        world
            .runtime
            .execute_prepared_plan(&next_plan, &(), &evaluator)?;
    }
    for _ in 0..=world.locality_definition().outputs().len() {
        let remaining = unsettled_nodes(world, &rerun)?;
        if remaining.is_empty() {
            break;
        }
        let next_plan = rerun.build_evaluation_plan(&remaining, EvaluationRequestMode::Default)?;
        rerun.execute_prepared_plan(&next_plan, &(), &evaluator)?;
    }
    for (output, expected) in fresh.shocked_values() {
        let node = world.handles[output];
        let restored_value = committed_value(world.runtime.graph(), node)?;
        let rerun_value = committed_value(&rerun, node)?;
        if restored_value != *expected || rerun_value != *expected {
            return Err(SignalError::internal(format!(
                "restored locality truth differs for {:?} at {output:?}: restored={restored_value}, rerun={rerun_value}, fresh={expected}",
                world.locality_definition().scale()
            )));
        }
    }
    let replay = world.runtime.replay_for_branch(analysis.id);
    if world.runtime.graph().captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::ReplayDetail,
    ) && (replay.frames.is_empty()
        || replay
            .frames
            .iter()
            .any(|frame| frame.branch_id != analysis.id))
    {
        let frame_summary = replay
            .frames
            .iter()
            .map(|frame| (frame.branch_id.0, frame.kind))
            .collect::<Vec<_>>();
        return Err(SignalError::internal(format!(
            "branch restore locality replay is not branch-local and restore-backed: analysis={} frames={frame_summary:?}",
            analysis.id.0,
        )));
    }
    Ok(FinancialRestoreLifecycleEvidence { _private: () })
}

fn unsettled_nodes(
    world: &CompiledFinancialLocalityWorld,
    graph: &SignalGraph,
) -> Result<Vec<crate::data::handle::NodeId>, SignalError> {
    world
        .locality_definition()
        .outputs()
        .iter()
        .map(|output| world.handles[&output.id])
        .filter(|node| {
            graph
                .get_state(*node)
                .is_ok_and(|state| state != NodeState::Clean)
        })
        .map(Ok)
        .collect()
}

fn publish_source(world: &mut CompiledFinancialLocalityWorld) -> Result<(), SignalError> {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mutations = world.locality_definition().action_traces()[0].committed_mutations();
    world.apply_mutations(&mutations)?;
    let fresh = FreshFinancialLocalityRecompute::run_for_trace(
        world.locality_definition(),
        &world.locality_definition().action_traces()[0],
    );
    let program = LocalityEvaluationProgram::shocked(
        world.locality_definition(),
        &world.handles,
        &world.baseline_values,
        fresh.shocked_values(),
        &mutations,
    );
    let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
    for producer in mutations
        .iter()
        .map(|mutation| mutation.producer)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let source = world.handles[&producer];
        world
            .runtime
            .transaction(request_execution, &mut (), |tx| {
                tx.read(source, &evaluator).map(|_| ())
            })?;
    }
    Ok(())
}

fn current_ready(
    graph: &mut SignalGraph,
    target: crate::data::handle::NodeId,
) -> Result<ReadyInvalidationBatch, SignalError> {
    let epoch = graph.begin_invalidation_readiness_epoch();
    let order = InvalidationStageOrder { stage: 0, order: 0 };
    let NodeInvalidationInput::Resolved(input) = graph.node_invalidation_input(target)? else {
        return Err(SignalError::internal(
            "restore target lacks resolved M12 authority",
        ));
    };
    let lowered = lower_current_work(graph, target, input, epoch, order)?;
    admit_current_readiness(graph, lowered, epoch, order)
}

fn require_semantic_causes(
    before: &[ResolvedDependencyCause],
    after: &[ResolvedDependencyCause],
) -> Result<(), SignalError> {
    let semantic = |cause: &ResolvedDependencyCause| {
        (
            cause.key.consumer,
            cause.key.producer,
            cause.key.aspect,
            cause.key.edge_scope.clone(),
            cause.key.dependency_revision,
            cause.binding_axes.cached_version,
            cause.binding_axes.output_commit_ordinal,
            cause.binding_axes.committed_version,
            cause.changed_scopes.clone(),
        )
    };
    if before.iter().map(semantic).eq(after.iter().map(semantic)) {
        Ok(())
    } else {
        Err(SignalError::internal(
            "restore changed canonical dependency cause identity",
        ))
    }
}

fn committed_value(
    graph: &SignalGraph,
    node: crate::data::handle::NodeId,
) -> Result<i64, SignalError> {
    graph
        .node_runtime_artifact_warm(node)?
        .and_then(|warm| warm.output_identity.as_ref())
        .and_then(|identity| identity.as_str().rsplit_once(':'))
        .and_then(|(_, value)| value.parse().ok())
        .ok_or_else(|| SignalError::internal("restored financial identity lacks a value"))
}

fn canonical_plan_order(
    plan: &crate::logic::planner::EvaluationPlan,
) -> Vec<(u32, Vec<crate::data::handle::NodeId>)> {
    plan.stages
        .iter()
        .map(|stage| {
            (
                stage.index,
                stage.tasks.iter().map(|task| task.node).collect(),
            )
        })
        .collect()
}
