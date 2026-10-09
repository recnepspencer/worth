use std::sync::atomic::{AtomicUsize, Ordering};

use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::{
    InvalidationReadinessEpoch, InvalidationStageOrder, ReadyInvalidationBatch,
};
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::data::telemetry::InvalidationPerformedCounter;
use crate::facade::{EvaluationRequestMode, NodeState};
use crate::logic::context::EvaluationContext;
use crate::logic::invalidation::scheduling::{
    admit_current_readiness, execute_ready, lower_current_work,
};
use crate::tests::domains::fintech::certification::invalidation::FreshFinancialLocalityRecompute;
use crate::tests::domains::fintech::world::{
    compile_financial_locality_world, ordinary_locality_cases, FinancialLocalityAction,
    FinancialLocalityScenario, FinancialWorldDefinition, LocalitySemanticOutputId,
};

use super::super::locality_evaluation::{runtime_shocked_values, LocalityEvaluationProgram};
use super::super::topology::signal_aspect;
use super::CompiledFinancialLocalityWorld;

#[test]
fn portfolio_dependency_churn_rejects_stale_same_shaped_ready_work() {
    let mut compiled = compile_scenario(FinancialLocalityScenario::PortfolioDependencyChurn);
    let world = compiled.locality_mut();
    let change = world.locality_definition().action_traces()[0]
        .actions()
        .iter()
        .find_map(|action| match action {
            FinancialLocalityAction::AcceptedOwnerMove { change, .. } => Some(*change),
            _ => None,
        })
        .unwrap();
    publish_source(world, change.before_subscription.upstream);
    let target = world.handles[&change.target];
    assert_eq!(
        world.runtime.graph().get_state(target).unwrap(),
        NodeState::Dirty
    );
    let ready = {
        let mut graph = world.runtime.graph_mut();
        current_ready(&mut graph, target)
    };

    let new_source = world.handles[&change.after_subscription.upstream];
    world
        .runtime
        .graph_mut()
        .set_dependencies(
            target,
            [DependencyEdge::new(
                new_source,
                signal_aspect(change.after_subscription.input_aspect),
            )],
        )
        .unwrap();
    let evaluations = AtomicUsize::new(0);
    let error = execute_ready(world.runtime.graph(), ready, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .expect_err("the old same-shaped ready binding must be stale after rewire");
    assert!(error.to_string().contains("stale dependency revision"));
    assert_eq!(evaluations.load(Ordering::SeqCst), 0);

    let risk = world.handles[&LocalitySemanticOutputId::new(3)];
    let retained_edges = world
        .runtime
        .graph()
        .dependencies_of(target)
        .unwrap()
        .to_vec();
    let retained_revision = world.runtime.graph().dependency_revision(target).unwrap();
    let counters = world.runtime.graph().invalidation_performed_counters();
    assert!(world
        .runtime
        .graph_mut()
        .set_dependencies(
            target,
            [DependencyEdge::new(
                risk,
                signal_aspect(change.after_subscription.input_aspect),
            )],
        )
        .is_err());
    assert_eq!(
        world.runtime.graph().dependencies_of(target).unwrap(),
        retained_edges
    );
    assert_eq!(
        world.runtime.graph().dependency_revision(target).unwrap(),
        retained_revision
    );
    let mut expected_after_rejection = counters.values();
    expected_after_rejection[InvalidationPerformedCounter::RejectedTopologyMutations as usize] += 1;
    assert_eq!(
        world.runtime.graph().invalidation_performed_counters(),
        crate::data::telemetry::SignalInvalidationRealizedCounters::from_values(
            expected_after_rejection,
        )
    );
}

#[test]
fn branch_restore_locality_replay_reconstitutes_work_from_m12_authority() {
    let mut compiled = compile_scenario(FinancialLocalityScenario::BranchRestoreLocalityReplay);
    let world = compiled.locality_mut();
    let mutation = world.locality_definition().mutation();
    publish_source(world, mutation.producer);
    let target = world
        .locality_definition()
        .outputs()
        .iter()
        .find(|output| !output.subscriptions.is_empty())
        .map(|output| world.handles[&output.id])
        .unwrap();
    let pre_restore_ready = {
        let mut graph = world.runtime.graph_mut();
        current_ready(&mut graph, target)
    };
    crate::facade::mark_dirty(
        world.runtime.graph_mut(),
        world.handles[&mutation.producer],
        signal_aspect(mutation.aspect),
    )
    .unwrap();
    let source_basis = world
        .runtime
        .graph()
        .get_entry(world.handles[&mutation.producer])
        .unwrap()
        .direct_invalidation_basis()
        .cloned()
        .unwrap();
    let causes_before = world
        .runtime
        .graph()
        .pending_causes(target)
        .unwrap()
        .to_vec();
    let image = crate::state::SignalCheckpointImage {
        authority: world.runtime.graph().capture_checkpoint_authority(),
        dependency_snapshot_batch: world
            .runtime
            .graph()
            .capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *world.runtime.graph().telemetry(),
    };
    let mut restored = SignalGraph::restore_from_checkpoint_image(&image).unwrap();
    let mut rerun = SignalGraph::restore_from_checkpoint_image(&image).unwrap();
    assert_eq!(restored.checkpoint_reconstruction_count(), 1);
    assert_eq!(
        restored
            .invalidation_performed_counters()
            .value(InvalidationPerformedCounter::RecoveryReconstructionWork),
        0
    );
    assert_ne!(restored.runtime_instance_id(), rerun.runtime_instance_id());

    assert!(execute_ready(&restored, pre_restore_ready, || Ok(()))
        .unwrap_err()
        .to_string()
        .contains("stale graph instance"));
    assert_eq!(
        restored
            .get_entry(world.handles[&mutation.producer])
            .unwrap()
            .direct_invalidation_basis(),
        Some(&source_basis)
    );
    assert_semantic_causes_equal(&causes_before, restored.pending_causes(target).unwrap());

    let rebuilt = current_ready(&mut restored, target);
    execute_ready(&restored, rebuilt, || Ok(())).unwrap();
    let shocked = runtime_shocked_values(
        world.locality_definition(),
        &world.baseline_values,
        &[mutation],
    )
    .unwrap();
    let program = LocalityEvaluationProgram::shocked(
        world.locality_definition(),
        &world.handles,
        &world.baseline_values,
        &shocked,
        &[mutation],
    );
    let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
    let plan = restored
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    let rerun_plan = rerun
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(
        canonical_plan_order(&plan),
        canonical_plan_order(&rerun_plan)
    );
    restored
        .execute_prepared_plan(&plan, &(), &evaluator)
        .unwrap();
    rerun
        .execute_prepared_plan(&rerun_plan, &(), &evaluator)
        .unwrap();
    let fresh = FreshFinancialLocalityRecompute::run(world.locality_definition());
    for (output, expected) in fresh.shocked_values() {
        assert_eq!(committed_value(&restored, world.handles[output]), *expected);
        assert_eq!(committed_value(&rerun, world.handles[output]), *expected);
    }
}

fn compile_scenario(
    scenario: FinancialLocalityScenario,
) -> crate::tests::domains::fintech::world::CompiledFinancialWorld {
    let case = ordinary_locality_cases()
        .into_iter()
        .find(|case| case.scenario() == scenario)
        .unwrap();
    compile_financial_locality_world(FinancialWorldDefinition::locality_case(41, case)).unwrap()
}

fn publish_source(world: &mut CompiledFinancialLocalityWorld, producer: LocalitySemanticOutputId) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mutation = world
        .locality_definition()
        .mutations()
        .iter()
        .copied()
        .find(|mutation| mutation.producer == producer)
        .unwrap_or_else(|| world.locality_definition().mutation());
    world.apply_mutations(&[mutation]).unwrap();
    let shocked = runtime_shocked_values(
        world.locality_definition(),
        &world.baseline_values,
        &[mutation],
    )
    .unwrap();
    let program = LocalityEvaluationProgram::shocked(
        world.locality_definition(),
        &world.handles,
        &world.baseline_values,
        &shocked,
        &[mutation],
    );
    let evaluator = |view: &mut EvaluationContext<'_, ()>| program.evaluate(view);
    let source = world.handles[&producer];
    world
        .runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &evaluator).map(|_| ())
        })
        .unwrap();
}

fn current_ready(
    graph: &mut SignalGraph,
    target: crate::data::handle::NodeId,
) -> ReadyInvalidationBatch {
    let epoch: InvalidationReadinessEpoch = graph.begin_invalidation_readiness_epoch();
    let order = InvalidationStageOrder { stage: 0, order: 0 };
    let NodeInvalidationInput::Resolved(input) = graph.node_invalidation_input(target).unwrap()
    else {
        panic!("financial target did not retain resolved M12 authority");
    };
    let lowered = lower_current_work(graph, target, input, epoch, order).unwrap();
    admit_current_readiness(graph, lowered, epoch, order).unwrap()
}

fn assert_semantic_causes_equal(
    before: &[crate::data::proof::invalidation::binding::ResolvedDependencyCause],
    after: &[crate::data::proof::invalidation::binding::ResolvedDependencyCause],
) {
    assert_eq!(before.len(), after.len());
    for (before, after) in before.iter().zip(after) {
        assert_eq!(before.key.producer, after.key.producer);
        assert_eq!(before.key.aspect, after.key.aspect);
        assert_eq!(before.key.edge_scope, after.key.edge_scope);
        assert_eq!(
            before.key.dependency_revision,
            after.key.dependency_revision
        );
        assert_eq!(
            before.binding_axes.output_commit_ordinal,
            after.binding_axes.output_commit_ordinal
        );
        assert_eq!(before.changed_scopes, after.changed_scopes);
    }
}

fn committed_value(graph: &SignalGraph, node: crate::data::handle::NodeId) -> i64 {
    graph
        .node_runtime_artifact_warm(node)
        .unwrap()
        .and_then(|warm| warm.output_identity.as_ref())
        .and_then(|identity| identity.as_str().rsplit_once(':'))
        .and_then(|(_, value)| value.parse().ok())
        .expect("financial output identity must contain its committed value")
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
