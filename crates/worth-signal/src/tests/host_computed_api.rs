use crate::data::dependency::DependencyEdge;
use crate::data::host_computed::{
    AdmittedHostComputedReadSet, HostComputedApiFamily, HostComputedDependencyPatch,
    HostComputedEvaluationOutcome, HostComputedFailureClass,
};
use crate::easy::SignalApp;
use crate::facade::{mark_dirty, Aspect, EvaluationRequestMode, NodeId, SignalGraph};
use crate::logic::context::EvaluationContext;
use crate::logic::prepared::{PreparedDependencyCapture, PreparedEvaluation};
use crate::tests::support::ASPECT_A;

#[test]
fn easy_computed_dynamic_dependencies_rewire_through_host_computed_admission() {
    let mut app = SignalApp::new();
    let enabled = app.input(true);
    let name = app.input(String::from("Ada"));
    let fallback = app.input(String::from("disabled"));
    let enabled_for_label = enabled;
    let name_for_label = name.clone();
    let fallback_for_label = fallback.clone();
    let label = app.computed(move |context| {
        if context.get(enabled_for_label) {
            context.get(name_for_label.clone())
        } else {
            context.get(fallback_for_label.clone())
        }
    });

    assert_eq!(app.get(label.clone()), "Ada");
    let initial = app.graph().dependencies_of(label.node).unwrap().to_vec();
    assert_eq!(
        initial,
        vec![
            DependencyEdge::new(enabled.node, ASPECT_A),
            DependencyEdge::new(name.node, ASPECT_A),
        ]
    );

    app.set(enabled, false);
    assert_eq!(app.get(label.clone()), "disabled");
    let rewired = app.graph().dependencies_of(label.node).unwrap().to_vec();
    assert_eq!(
        rewired,
        vec![
            DependencyEdge::new(enabled.node, ASPECT_A),
            DependencyEdge::new(fallback.node, ASPECT_A),
        ]
    );
}

#[test]
fn host_computed_dependency_patch_can_be_built_from_admitted_reads() {
    let node = NodeId::new(10, 0);
    let source_a = NodeId::new(11, 0);
    let source_b = NodeId::new(12, 0);
    let source_c = NodeId::new(13, 0);
    let mut capture = PreparedDependencyCapture::new();
    capture.record(source_a, Aspect::new(0), None);
    capture.record(source_c, Aspect::new(0), None);
    let admitted = AdmittedHostComputedReadSet::admit(node, capture).unwrap();

    let patch = HostComputedDependencyPatch::between(
        node,
        &[
            DependencyEdge::new(source_a, Aspect::new(0)),
            DependencyEdge::new(source_b, Aspect::new(0)),
        ],
        &admitted,
    );

    assert_eq!(patch.node(), node);
    assert_eq!(patch.retained_dependency_count(), 1);
    assert_eq!(
        patch.added_dependencies(),
        &[DependencyEdge::new(source_c, Aspect::new(0))]
    );
    assert_eq!(
        patch.removed_dependencies(),
        &[DependencyEdge::new(source_b, Aspect::new(0))]
    );
}

#[test]
fn generic_execute_prepared_plan_rewires_dynamic_dependencies_through_host_admission() {
    let mut graph = SignalGraph::new();
    let enabled = graph.node().build();
    let left = graph.node().build();
    let right = graph.node().build();
    let target = graph.node().on_demand().build();
    graph
        .set_dependencies(target, [DependencyEdge::new(enabled, ASPECT_A)])
        .unwrap();

    let mut use_left = true;
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    crate::logic::planner::execute_prepared_plan(
        &mut graph,
        &plan,
        &(),
        &|ctx: &mut EvaluationContext<'_, ()>| {
            if ctx.node() == target {
                let _ = ctx.read(enabled, ASPECT_A)?;
                if use_left {
                    let _ = ctx.read(left, ASPECT_A)?;
                } else {
                    let _ = ctx.read(right, ASPECT_A)?;
                }
            }
            Ok(ctx.finish(crate::tests::support::version_ab(1, 0)))
        },
    )
    .unwrap();

    assert_eq!(
        graph.dependencies_of(target).unwrap(),
        &[
            DependencyEdge::new(enabled, ASPECT_A),
            DependencyEdge::new(left, ASPECT_A),
        ]
    );

    use_left = false;
    mark_dirty(&mut graph, enabled, ASPECT_A).unwrap();
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    crate::logic::planner::execute_prepared_plan(
        &mut graph,
        &plan,
        &(),
        &|ctx: &mut EvaluationContext<'_, ()>| {
            if ctx.node() == target {
                let _ = ctx.read(enabled, ASPECT_A)?;
                if use_left {
                    let _ = ctx.read(left, ASPECT_A)?;
                } else {
                    let _ = ctx.read(right, ASPECT_A)?;
                }
            }
            Ok(ctx.finish(crate::tests::support::version_ab(2, 0)))
        },
    )
    .unwrap();

    assert_eq!(
        graph.dependencies_of(target).unwrap(),
        &[
            DependencyEdge::new(enabled, ASPECT_A),
            DependencyEdge::new(right, ASPECT_A),
        ]
    );
}

#[test]
fn generic_execute_prepared_plan_denies_self_reads_through_host_admission() {
    let mut graph = SignalGraph::new();
    let target = graph.node().on_demand().build();
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();

    let err = crate::logic::planner::execute_prepared_plan(
        &mut graph,
        &plan,
        &(),
        &|ctx: &mut EvaluationContext<'_, ()>| {
            let _ = ctx.read(target, ASPECT_A)?;
            Ok(ctx.finish(crate::tests::support::version_ab(1, 0)))
        },
    )
    .unwrap_err();

    assert!(format!("{err}").contains("host-computed read admission denied"));
    assert!(format!("{err}").contains("SelfRead"));
}

#[test]
fn generic_execute_prepared_plan_records_host_computed_telemetry() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let target = graph.node().on_demand().build();
    let before = graph.observe().metrics().host_computed;
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();

    crate::logic::planner::execute_prepared_plan(
        &mut graph,
        &plan,
        &(),
        &|ctx: &mut EvaluationContext<'_, ()>| {
            if ctx.node() == target {
                let _ = ctx.read(source, ASPECT_A)?;
            }
            Ok(ctx.finish(crate::tests::support::version_ab(1, 0)))
        },
    )
    .unwrap();

    let after = graph.observe().metrics().host_computed;
    assert_eq!(
        after.descriptor_registration_count - before.descriptor_registration_count,
        1
    );
    assert_eq!(
        after.evaluation_request_admission_count - before.evaluation_request_admission_count,
        1
    );
    assert_eq!(
        after.read_set_admission_count - before.read_set_admission_count,
        1
    );
    assert_eq!(
        after.dependency_patch_count - before.dependency_patch_count,
        1
    );
    assert_eq!(
        after.dependency_patch_added_count - before.dependency_patch_added_count,
        1
    );
}

#[test]
fn host_computed_telemetry_is_idle_on_demand_and_captured_by_explicit_session() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(crate::facade::SignalRuntimePolicy::operational());
    let source = graph.node().build();
    let target = graph.node().on_demand().build();
    let before = graph.observe().metrics().host_computed;
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    crate::logic::planner::execute_prepared_plan(
        &mut graph,
        &plan,
        &(),
        &|ctx: &mut EvaluationContext<'_, ()>| {
            if ctx.node() == target {
                let _ = ctx.read(source, ASPECT_A)?;
            }
            Ok(ctx.finish(crate::tests::support::version_ab(1, 0)))
        },
    )
    .unwrap();
    assert_eq!(graph.observe().metrics().host_computed, before);

    let session = graph
        .begin_observation_session(crate::facade::SignalObservationRequest::telemetry())
        .unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    crate::logic::planner::execute_prepared_plan(
        &mut graph,
        &plan,
        &(),
        &|ctx: &mut EvaluationContext<'_, ()>| {
            if ctx.node() == target {
                let _ = ctx.read(source, ASPECT_A)?;
            }
            Ok(ctx.finish(crate::tests::support::version_ab(2, 0)))
        },
    )
    .unwrap();
    let after = graph.observe().metrics().host_computed;
    assert!(after.evaluation_request_admission_count > before.evaluation_request_admission_count);
    assert!(after.committed_artifact_count > before.committed_artifact_count);
    graph.cancel_observation_session(&session).unwrap();
}

#[test]
fn host_computed_outcomes_expose_typed_diagnostics_summary() {
    let node = NodeId::new(40, 0);
    let source = NodeId::new(41, 0);
    let mut capture = PreparedDependencyCapture::new();
    capture.record(source, Aspect::new(0), None);
    let prepared = PreparedEvaluation::from_result(
        crate::facade::NodeEvaluationResult::from_version(crate::facade::AspectVersion::zero()),
    )
    .with_dependencies(capture);
    let admitted = crate::data::host_computed::PreparedHostComputedEvaluation::admit(
        crate::data::host_computed::HostComputedEvaluationRequest::new(
            crate::data::host_computed::HostComputedDescriptor::for_node(
                node,
                HostComputedApiFamily::CorePreparedEvaluation,
            ),
            &[],
        ),
        prepared,
    )
    .unwrap();

    let committed = HostComputedEvaluationOutcome::committed(admitted.clone());
    let committed_summary = committed.diagnostics_summary();
    assert_eq!(
        committed_summary.outcome(),
        crate::data::host_computed::HostComputedOutcomeClass::Committed
    );
    assert_eq!(committed_summary.admitted_read_count(), 1);

    let denied = HostComputedEvaluationOutcome::denied(
        admitted.request().clone(),
        crate::data::host_computed::DeniedHostComputedReadSet::new(
            node,
            crate::data::host_computed::HostComputedDenialClass::SelfRead,
            DependencyEdge::new(node, ASPECT_A),
        ),
    );
    let denied_summary = denied.diagnostics_summary();
    assert_eq!(
        denied_summary.outcome(),
        crate::data::host_computed::HostComputedOutcomeClass::Denied
    );
    assert_eq!(
        denied_summary.denial_class(),
        Some(crate::data::host_computed::HostComputedDenialClass::SelfRead)
    );

    let failed = HostComputedEvaluationOutcome::failed(
        admitted.descriptor().clone(),
        HostComputedFailureClass::HostAdapterRejected,
        "callback unavailable",
    );
    let failed_summary = failed.diagnostics_summary();
    assert_eq!(
        failed_summary.outcome(),
        crate::data::host_computed::HostComputedOutcomeClass::Failed
    );
    assert_eq!(failed_summary.failure_class(), Some("HostAdapterRejected"));
}
