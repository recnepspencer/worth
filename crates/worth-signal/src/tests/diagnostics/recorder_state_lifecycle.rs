use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::subscriber_context::SubscriberContext;
use crate::facade::*;
use crate::state::SignalSnapshotDiagnostics;
use crate::tests::support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsEvent {
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DiagnosticsDomain {
    Cache,
}

struct FailingSubscriber;
impl EventSubscriber for FailingSubscriber {
    type Event = DiagnosticsEvent;
    type DataId = DiagnosticsDomain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(99)
    }

    fn name(&self) -> &'static str {
        "failing"
    }

    fn requires(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn provides(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn on_event(&mut self, _event: &Self::Event) {}

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        Err(SignalError::internal("injected subscriber failure"))
    }
}

struct NeedsMissingProviderSubscriber;
impl EventSubscriber for NeedsMissingProviderSubscriber {
    type Event = DiagnosticsEvent;
    type DataId = DiagnosticsDomain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(100)
    }

    fn name(&self) -> &'static str {
        "missing-provider"
    }

    fn requires(&self) -> &'static [Self::DataId] {
        &[DiagnosticsDomain::Cache]
    }

    fn provides(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn on_event(&mut self, _event: &Self::Event) {}

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        Ok(())
    }
}

#[test]
fn restore_snapshot_payload_preserving_history_keeps_latest_observation_in_sync_with_flow() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(2, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    let report = graph.execute_prepared_plan(&plan, &(), &compute).unwrap();

    let payload_observation = ObservationBoundarySummary {
        classified_event_count: 1,
        trigger_matched_event_count: 1,
        delivered_event_count: 1,
        rollback_suppressed_event_count: 0,
        branch_local_suppressed_event_count: 0,
        boundary_events: Vec::new(),
    };
    let current_observation = ObservationBoundarySummary {
        classified_event_count: 2,
        trigger_matched_event_count: 1,
        delivered_event_count: 0,
        rollback_suppressed_event_count: 1,
        branch_local_suppressed_event_count: 0,
        boundary_events: Vec::new(),
    };

    let mut payload_flow = FlowSummary::new(
        DiagnosticsTier::Development,
        ChangeInputSummary::new(vec![node], vec![ASPECT_A], 0, None),
        InvalidationSummary::new(1, 0, 0, 0, 0),
        PlanningSummary::from_plan(&plan, DiagnosticsTier::Development),
        PrecomputeSummary::from_report(&report, DiagnosticsTier::Development),
        ApplySummary::from_report(&report, DiagnosticsTier::Development),
        Vec::new(),
        Vec::new(),
        Some(payload_observation.clone()),
        None,
        None,
    );
    payload_flow.observation = Some(payload_observation.clone());

    let payload = SignalSnapshotDiagnostics {
        latest_flow: Some(payload_flow),
        latest_failure: None,
        latest_rollback: None,
        latest_observation: Some(payload_observation),
        recent_history: Default::default(),
        replay_frames: Default::default(),
        explanation_facts: Default::default(),
        provenance_facts: Default::default(),
        lineage_records: Default::default(),
        branch_catalog: graph
            .diagnostics_state()
            .branch_catalog()
            .iter()
            .map(|(id, handle)| (*id, handle.clone()))
            .collect(),
        active_branch: graph.diagnostics_state().active_branch().id,
        next_replay_cursor: 0,
        next_snapshot_id: 0,
        next_branch_id: 1,
        next_lineage_artifact_id: 0,
        next_lineage_sequence: 0,
        observation_activation_mask: 0,
    };

    graph
        .diagnostics_state_mut()
        .record_observation(current_observation.clone());
    let current = graph.diagnostics_state().clone();
    graph
        .diagnostics_state_mut()
        .restore_snapshot_payload_preserving_history_from(payload, &current);

    assert_eq!(
        graph.diagnostics_state().latest_observation(),
        Some(&current_observation)
    );
    assert_eq!(
        graph
            .diagnostics_state()
            .latest_flow()
            .and_then(|flow| flow.observation),
        Some(&current_observation)
    );
}

#[test]
fn successful_execution_automatically_records_flow_and_history_diagnostics() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let source_compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));
    let dependent_compute = |ctx: &mut EvaluationContext<'_, ()>| {
        let version = ctx.read_aspect_version(source, ASPECT_A)?;
        Ok(ctx.finish(NodeEvaluationResult::from_version(version)))
    };

    let bootstrap = graph
        .build_evaluation_plan(&[source, dependent], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            if ctx.node() == source {
                source_compute(ctx)
            } else {
                dependent_compute(ctx)
            }
        })
        .unwrap();

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let plan = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|ctx| {
            if ctx.node() == source {
                source_compute(ctx)
            } else {
                dependent_compute(ctx)
            }
        })
        .unwrap();

    let flow = graph
        .observe()
        .latest_flow_diagnostics()
        .expect("flow diagnostics should be recorded");
    let frontier = graph
        .observe()
        .latest_frontier_execution_summary()
        .expect("frontier execution summary should be retained");
    assert_eq!(flow.change.changed_nodes, vec![source]);
    assert_eq!(flow.planning.plan.task_count, 2);
    assert_eq!(flow.apply.report.task_count, 2);
    assert_eq!(
        flow.invalidation.frontier_seed_count as u64,
        frontier.counters.frontier_seed_count
    );
    assert_eq!(
        flow.invalidation.frontier_direct_wave_count as u64,
        frontier.counters.frontier_direct_wave_count
    );
    assert_eq!(
        flow.invalidation.frontier_transitive_wave_count as u64,
        frontier.counters.frontier_transitive_wave_count
    );
    assert!(!graph
        .observe()
        .recent_execution_history_diagnostics()
        .is_empty());
}

#[test]
fn execution_failures_and_rollbacks_automatically_record_diagnostics() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    let err = runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.evaluate_with_plan(
                node,
                &|_view| {
                    Err::<crate::logic::evaluation::EvaluationOutput, _>(SignalError::internal(
                        "synthetic precompute failure",
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap_err();
    assert!(format!("{err}").contains("synthetic precompute failure"));

    let failure = runtime
        .observe()
        .latest_failure_diagnostics()
        .expect("failure diagnostics should be retained");
    assert_eq!(failure.phase, ExecutionFailurePhase::Precompute);

    let rollback = runtime
        .observe()
        .latest_rollback_diagnostics()
        .expect("rollback diagnostics should be retained");
    assert!(rollback.rolled_back);
}

#[test]
fn repeated_rollbacks_keep_latest_rollback_current_and_bounded() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime
        .graph_mut()
        .reset_runtime_policy_to_tier(DiagnosticsTier::Development);
    let node = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    for _ in 0..100 {
        let err = runtime.transaction(&mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            Err(SignalError::invalid_input("force rollback"))
        });
        assert!(err.is_err());
    }

    let diagnostics = runtime.observe().diagnostics();
    let rollback = diagnostics
        .latest_rollback()
        .expect("latest rollback should be retained");
    assert!(rollback.rolled_back);
    assert!(rollback
        .reason
        .as_deref()
        .unwrap_or_default()
        .contains("explicit rollback"));
    assert!(
        diagnostics.recent_history().len()
            <= SignalRuntimePolicy::for_tier(DiagnosticsTier::Development)
                .retention_budget
                .history_limit
    );
}

#[test]
fn commit_promotion_failures_record_failure_and_rollback_diagnostics() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_events::<DiagnosticsEvent>()
        .with_domains::<DiagnosticsDomain>()
        .build();
    let node = runtime.graph_mut().node().build();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingSubscriber))
        .unwrap();

    let mut runtime_ctx = ();
    let err = runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.emit_event(DiagnosticsEvent::Tick);
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap_err();
    assert!(format!("{err}").contains("event bus flush failed"));

    let failure = runtime
        .observe()
        .latest_failure_diagnostics()
        .expect("flush failure diagnostics should be retained");
    assert_eq!(failure.phase, ExecutionFailurePhase::CommitPromotion);
    let rollback = runtime
        .observe()
        .latest_rollback_diagnostics()
        .expect("flush rollback diagnostics should be retained");
    assert!(rollback.rolled_back);
    assert!(rollback
        .reason
        .as_deref()
        .unwrap_or_default()
        .contains("event bus flush failed"));
}

#[test]
fn event_bus_begin_failures_record_failure_and_rollback_diagnostics() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_events::<DiagnosticsEvent>()
        .with_domains::<DiagnosticsDomain>()
        .build();
    let node = runtime.graph_mut().node().build();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(NeedsMissingProviderSubscriber))
        .unwrap();

    let mut runtime_ctx = ();
    let err = runtime
        .transaction(&mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            Ok(())
        })
        .unwrap_err();
    assert!(format!("{err}").contains("event bus begin failed"));

    let failure = runtime
        .observe()
        .latest_failure_diagnostics()
        .expect("begin failure diagnostics should be retained");
    assert_eq!(failure.phase, ExecutionFailurePhase::CommitPromotion);
    let rollback = runtime
        .observe()
        .latest_rollback_diagnostics()
        .expect("begin rollback diagnostics should be retained");
    assert!(rollback.rolled_back);
    assert!(rollback
        .reason
        .as_deref()
        .unwrap_or_default()
        .contains("event bus begin failed"));
}
