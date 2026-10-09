use crate::data::checkpoint::CheckpointBarrier;
use crate::data::dirty_set::DomainImpact;
use crate::data::error::SignalError;
use crate::data::evaluator::CheckpointEvaluator;
use crate::data::graph::ScratchLeaseKind;
use crate::facade::{
    DependencyEdge, SignalGraph, SignalObservationRequest, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::ASPECT_A;

struct EmptyCheckpointEvaluator;

impl CheckpointEvaluator for EmptyCheckpointEvaluator {
    type Domain = ();
    type Impact = ();
    type Context = ();

    fn refresh(
        &mut self,
        _domain: Self::Domain,
        _impact: DomainImpact<Self::Impact>,
        _ctx: &mut Self::Context,
    ) -> Result<(), SignalError> {
        Ok(())
    }
}

#[test]
fn on_demand_idle_does_not_capture_structural_runtime_telemetry() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    let source = graph.node().build();
    let consumer = graph.node().build();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(source, ASPECT_A)])
        .unwrap();

    let bootstrap = graph
        .build_evaluation_plan(
            &[consumer],
            crate::logic::evaluation::EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|context| {
            Ok(context.finish(crate::tests::support::version_ab(1, 0)))
        })
        .unwrap();
    crate::facade::mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let plan = graph
        .build_evaluation_plan(
            &[consumer],
            crate::logic::evaluation::EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|context| {
            Ok(context.finish(crate::tests::support::version_ab(2, 0)))
        })
        .unwrap();

    assert_eq!(
        *graph.observe().telemetry(),
        crate::data::telemetry::RuntimeTelemetry::default(),
        "structural telemetry must remain idle until OptionalTelemetry is admitted"
    );
}

#[test]
fn on_demand_gc_epoch_does_not_capture_structural_runtime_telemetry() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    graph.set_gc_compaction_state_for_test(2, 0);

    graph.run_gc_epoch();

    assert_eq!(
        graph.observe().telemetry().storage.gc_epoch_count,
        0,
        "GC telemetry must remain idle when OptionalTelemetry is not admitted"
    );
    assert_eq!(graph.observe().telemetry().storage.gc_epoch_nanos, 0);
}

#[test]
fn explicit_telemetry_session_captures_gc_epoch_telemetry() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    let session = graph
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    graph.set_gc_compaction_state_for_test(2, 0);

    graph.run_gc_epoch();

    assert_eq!(graph.observe().telemetry().storage.gc_epoch_count, 1);
    assert!(graph.observe().telemetry().storage.gc_epoch_nanos > 0);
    graph.cancel_observation_session(&session).unwrap();
}

#[test]
fn on_demand_nested_scratch_does_not_capture_structural_runtime_telemetry() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    let scratch = graph.acquire_scratch(ScratchLeaseKind::Evaluation).unwrap();

    assert!(graph
        .acquire_scratch(ScratchLeaseKind::Invalidation)
        .is_err());
    assert_eq!(
        graph
            .observe()
            .telemetry()
            .storage
            .scratch_reentry_error_count,
        0,
        "scratch reentry telemetry must remain idle when OptionalTelemetry is not admitted"
    );
    graph
        .restore_scratch(ScratchLeaseKind::Evaluation, scratch)
        .unwrap();
}

#[test]
fn explicit_telemetry_session_captures_nested_scratch_telemetry() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    let session = graph
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    let scratch = graph.acquire_scratch(ScratchLeaseKind::Evaluation).unwrap();

    assert!(graph
        .acquire_scratch(ScratchLeaseKind::Invalidation)
        .is_err());
    assert_eq!(
        graph
            .observe()
            .telemetry()
            .storage
            .scratch_reentry_error_count,
        1
    );
    graph
        .restore_scratch(ScratchLeaseKind::Evaluation, scratch)
        .unwrap();
    graph.cancel_observation_session(&session).unwrap();
}

#[test]
fn on_demand_signal_runtime_does_not_absorb_execution_report_telemetry() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let source = runtime.graph_mut().node().build();
    let target = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .set_dependencies(target, [DependencyEdge::new(source, ASPECT_A)])
        .unwrap();
    let bootstrap = runtime
        .graph_mut()
        .build_evaluation_plan(
            &[target],
            crate::logic::evaluation::EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
    runtime
        .graph_mut()
        .execute_prepared_plan(&bootstrap, &(), &|context| {
            Ok(context.finish(crate::tests::support::version_ab(1, 0)))
        })
        .unwrap();
    let idle_before = *runtime.telemetry();
    crate::facade::mark_dirty(runtime.graph_mut(), target, ASPECT_A).unwrap();
    runtime
        .evaluate_dirty(&(), &|context| {
            Ok(context.finish(crate::tests::support::version_ab(2, 0)))
        })
        .unwrap();

    assert_eq!(
        *runtime.telemetry(),
        idle_before,
        "SignalRuntime must not absorb any optional execution-report telemetry without admission"
    );

    let session = runtime
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    crate::facade::mark_dirty(runtime.graph_mut(), target, ASPECT_A).unwrap();
    runtime
        .evaluate_dirty(&(), &|context| {
            Ok(context.finish(crate::tests::support::version_ab(3, 0)))
        })
        .unwrap();
    assert_ne!(*runtime.telemetry(), idle_before);
    assert!(runtime.telemetry().planner.plans_built > idle_before.planner.plans_built);
    assert!(
        runtime.telemetry().execution.stage_execution_count
            > idle_before.execution.stage_execution_count
    );
    runtime.cancel_observation_session(&session).unwrap();
}

#[test]
fn on_demand_branch_lifecycle_does_not_capture_runtime_telemetry() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let idle = *runtime.telemetry();

    let _basis = runtime.current_branch_basis_artifact();
    let child = runtime.create_branch("telemetry-idle-child").unwrap();
    runtime.switch_branch(child).unwrap();

    assert_eq!(
        *runtime.telemetry(),
        idle,
        "branch basis/fork/transfer telemetry must remain idle without admission"
    );

    let session = runtime
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    let _basis = runtime.current_branch_basis_artifact();
    let observed = *runtime.telemetry();
    assert!(
        observed.transaction.branch_basis_production_count
            > idle.transaction.branch_basis_production_count
    );
    runtime.cancel_observation_session(&session).unwrap();
}

#[test]
fn on_demand_async_capability_declaration_does_not_capture_resource_telemetry() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let node = runtime.graph_mut().node().build();
    let declaration = crate::facade::AsyncNodeCapabilityDeclaration::new(
        node,
        crate::facade::AsyncNodePayloadContract::new(
            crate::facade::AsyncNodePayloadContractId::new(91),
        )
        .with_max_payload_bytes(1024),
    );
    let idle = *runtime.telemetry();
    runtime.declare_async_node_capability(declaration).unwrap();
    assert_eq!(
        *runtime.telemetry(),
        idle,
        "async capability declaration must not capture resource telemetry without admission"
    );
}

#[test]
fn explicit_telemetry_session_captures_async_resource_telemetry() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let node = runtime.graph_mut().node().build();
    let declaration = crate::facade::AsyncNodeCapabilityDeclaration::new(
        node,
        crate::facade::AsyncNodePayloadContract::new(
            crate::facade::AsyncNodePayloadContractId::new(92),
        )
        .with_max_payload_bytes(1024),
    );
    let session = runtime
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    runtime.declare_async_node_capability(declaration).unwrap();

    assert!(
        runtime
            .telemetry()
            .resource
            .async_node_capability_attachment_count
            > 0
    );
    assert!(
        runtime
            .telemetry()
            .resource
            .resource_declaration_lowering_count
            > 0
    );
    runtime.cancel_observation_session(&session).unwrap();
}

#[test]
fn on_demand_transaction_event_and_checkpoint_telemetry_stay_idle() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    type TestRuntime = SignalRuntime<(), (), (), (), ()>;
    let mut runtime: TestRuntime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let mut ctx = ();
    let mut eval_ctx = ();
    let mut evaluator = EmptyCheckpointEvaluator;
    let mut transaction = runtime.begin(request_execution, &mut ctx);
    transaction.emit_event(());
    transaction
        .flush_events(CheckpointBarrier::PerOperation)
        .unwrap();
    transaction
        .flush_checkpoint(
            CheckpointBarrier::PerOperation,
            &mut evaluator,
            &mut eval_ctx,
        )
        .unwrap();
    transaction.commit().unwrap();

    assert_eq!(runtime.event_bus().telemetry().checkpoint.event_flushes, 0);
    assert_eq!(runtime.event_bus().telemetry().checkpoint.rollback_count, 0);
    assert_eq!(
        runtime
            .checkpoint()
            .telemetry()
            .checkpoint
            .checkpoint_flushes,
        0
    );
    assert_eq!(
        runtime
            .checkpoint()
            .telemetry()
            .checkpoint
            .checkpoint_flush_nanos,
        0
    );
}

#[path = "runtime_telemetry/explicit_telemetry_session_captures_transaction_event_and_checkpoint_telemetry.rs"]
mod explicit_telemetry_session_captures_transaction_event_and_checkpoint_telemetry;
