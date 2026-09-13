use crate::facade::{
    ArtifactRetentionPolicy, DiagnosticsAvailability, EvaluationRequestMode, NodeEvaluationResult,
    NodeId, SignalGraph, SignalObservationRequest, SignalRuntimePolicy,
};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A};

const ACCESS_COUNTER_CHILD_ENV: &str = "WORTH_SIGNAL_ACCESS_COUNTER_CHILD";
const ACCESS_COUNTER_TEST_NAME: &str =
    "tests::observability::access_counters::artifact_access_counters_attribute_lane_api_and_denial_reason";

#[test]
fn artifact_access_counters_attribute_lane_api_and_denial_reason() {
    if super::isolated_counter_process::run_in_isolated_counter_process(
        ACCESS_COUNTER_TEST_NAME,
        ACCESS_COUNTER_CHILD_ENV,
    ) {
        return;
    }

    let mut retained_graph = SignalGraph::new();
    let retained_source = retained_graph.node().build();
    let retained_dependent = retained_graph.node().build();
    retained_graph
        .append_dependency(retained_dependent, retained_source, ASPECT_A)
        .unwrap();
    retained_graph.set_runtime_policy(SignalRuntimePolicy::development());
    let retained_bootstrap = retained_graph
        .build_evaluation_plan(
            &[retained_source, retained_dependent],
            EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
    retained_graph
        .execute_prepared_plan_with_precompute(&retained_bootstrap, &|node, view| {
            let result = if node == retained_source {
                view.finish(version_ab(1, 0))
            } else {
                let version = view.read_aspect_version(retained_source, ASPECT_A)?;
                view.finish(NodeEvaluationResult::from_version(version))
            };
            Ok(result)
        })
        .unwrap();
    let access_before_explanation = crate::data::access_counters::snapshot();
    let observed_explanation = retained_graph
        .observe()
        .explain(retained_dependent)
        .unwrap();
    let explanation_access =
        crate::data::access_counters::snapshot().delta_since(access_before_explanation);
    assert_eq!(explanation_access.materialized_entry_reads, 0);
    assert_eq!(explanation_access.materialized_entry_writes, 0);
    let retained_explanation = retained_graph
        .observe()
        .materialize()
        .retained_explanation_artifact(retained_dependent)
        .expect("development policy must retain the dependent explanation");
    assert_eq!(observed_explanation.state, retained_explanation.state);
    let retained_validation = retained_graph
        .node_explanation_storage_view(retained_dependent)
        .unwrap();
    assert_eq!(retained_validation.state(), retained_explanation.state);
    assert!(
        retained_validation.matches_historical_artifact_record(
            retained_explanation.historical_artifact_record.as_ref(),
        ),
        "component validation must preserve the retained explanation's current graph facts"
    );
    assert!(retained_graph
        .observe()
        .materialize()
        .retained_provenance_artifact(retained_dependent)
        .is_some());
    let retained_metrics = retained_graph.observe().metrics();
    assert_eq!(retained_metrics.storage.retained_forensic_read_count, 2);
    assert!(
        retained_metrics.storage.retained_artifact_read_count >= 2,
        "retained artifact reads should include the two explicit retained forensic fetches"
    );
    assert!(
        retained_metrics
            .storage
            .explicit_cold_materialization_request_count
            <= 1,
        "retained artifact access should not pay reconstructed-style explicit materialization costs"
    );

    let mut reconstructed_graph = SignalGraph::new();
    let reconstructed_source = reconstructed_graph.node().build();
    let reconstructed_dependent = reconstructed_graph.node().build();
    reconstructed_graph
        .append_dependency(reconstructed_dependent, reconstructed_source, ASPECT_A)
        .unwrap();
    reconstructed_graph.set_runtime_policy(SignalRuntimePolicy::operational());
    let session = reconstructed_graph
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    reconstructed_graph
        .cancel_observation_session(&session)
        .unwrap();
    let mut reconstructed_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(
        &mut reconstructed_graph,
        reconstructed_source,
        &mut reconstructed_compute,
    )
    .unwrap();
    evaluate(
        &mut reconstructed_graph,
        reconstructed_dependent,
        &mut reconstructed_compute,
    )
    .unwrap();
    reconstructed_graph
        .materialize_explanation_artifact(reconstructed_dependent)
        .unwrap();
    reconstructed_graph
        .materialize_provenance_artifact(reconstructed_dependent)
        .unwrap();
    let reconstructed_metrics = reconstructed_graph.observe().metrics();
    assert_eq!(
        reconstructed_metrics
            .storage
            .explicit_cold_materialization_request_count,
        2
    );
    assert_eq!(
        reconstructed_metrics
            .storage
            .cold_explanation_reconstruction_count,
        1
    );
    assert_eq!(
        reconstructed_metrics
            .storage
            .cold_provenance_reconstruction_count,
        1
    );
    assert_eq!(
        reconstructed_metrics
            .storage
            .reconstructed_artifact_read_count,
        2
    );

    let mut omitted_graph = SignalGraph::new();
    let omitted_source = omitted_graph.node().build();
    let omitted_dependent = omitted_graph.node().build();
    omitted_graph
        .append_dependency(omitted_dependent, omitted_source, ASPECT_A)
        .unwrap();
    omitted_graph.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_explanation_retention(ArtifactRetentionPolicy::Omit)
            .with_provenance_retention(ArtifactRetentionPolicy::Omit),
    );
    let session = omitted_graph
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    omitted_graph.cancel_observation_session(&session).unwrap();
    let mut omitted_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut omitted_graph, omitted_source, &mut omitted_compute).unwrap();
    evaluate(&mut omitted_graph, omitted_dependent, &mut omitted_compute).unwrap();
    assert_eq!(
        omitted_graph
            .materialize_explanation_artifact(omitted_dependent)
            .unwrap()
            .1,
        DiagnosticsAvailability::OmittedByTier
    );
    assert_eq!(
        omitted_graph
            .materialize_provenance_artifact(omitted_dependent)
            .unwrap()
            .1,
        DiagnosticsAvailability::OmittedByTier
    );
    let omitted_metrics = omitted_graph.observe().metrics();
    assert_eq!(
        omitted_metrics.storage.denied_reconstruction_by_tier_count,
        2
    );
    assert_eq!(
        omitted_metrics
            .storage
            .denied_reconstruction_explanation_api_count,
        1
    );
    assert_eq!(
        omitted_metrics
            .storage
            .denied_reconstruction_provenance_api_count,
        1
    );

    let mut denied_graph = SignalGraph::new();
    let denied_source = denied_graph.node().build();
    let denied_dependent = denied_graph.node().build();
    denied_graph
        .append_dependency(denied_dependent, denied_source, ASPECT_A)
        .unwrap();
    let mut denied_policy = SignalRuntimePolicy::operational();
    denied_policy
        .reconstruction_budget
        .allow_explanation_reconstruction = false;
    denied_policy
        .reconstruction_budget
        .allow_provenance_reconstruction = false;
    denied_graph.set_runtime_policy(denied_policy);
    let session = denied_graph
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    denied_graph.cancel_observation_session(&session).unwrap();
    let mut denied_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut denied_graph, denied_source, &mut denied_compute).unwrap();
    evaluate(&mut denied_graph, denied_dependent, &mut denied_compute).unwrap();
    assert_eq!(
        denied_graph
            .materialize_explanation_artifact(denied_dependent)
            .unwrap()
            .1,
        DiagnosticsAvailability::DeniedByBudget
    );
    assert_eq!(
        denied_graph
            .materialize_provenance_artifact(denied_dependent)
            .unwrap()
            .1,
        DiagnosticsAvailability::DeniedByBudget
    );
    let denied_metrics = denied_graph.observe().metrics();
    assert_eq!(
        denied_metrics.storage.denied_reconstruction_by_budget_count,
        2
    );
    assert_eq!(
        denied_metrics
            .storage
            .explicit_cold_materialization_request_count,
        2
    );
}
