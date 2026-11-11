//! World baseline transport preserves the actual registered Bridge source.
use worth_runtime_bridge::facade::*;
use worth_signal::facade::SignalGraph;
mod declaration;

#[test]
fn world_baseline_export_preserves_installed_semantic_correspondences() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let candidate = declaration::dependency();
    let bridge = declaration::runtime(candidate.clone(), &graph, node);
    let port = bridge.runtime_world_correspondence_port();
    let baseline = port
        .admit_runtime_baseline(graph.installed_graph_capability().graph_instance_id())
        .unwrap();
    let policy = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let request = worth_execution::ExecutionRequest::serial(&policy);
    let recorded = port
        .export_recorded_runtime_world_baseline_in(&request, &baseline)
        .unwrap();
    let expected = RecordedRuntimeWorldBaseline::from_json_in(&request,
        br#"{"version":2,"baseline":"bridge-runtime-world-baseline-v2","adapter_semantic_identity":"relational-adapter:99","graph_role":"primary"}"#).unwrap();
    assert_eq!(recorded, expected);
    let installed = bridge
        .bind_signal_graph(&mut graph)
        .unwrap()
        .install_semantic_correspondence(candidate)
        .unwrap();
    let admitted = port.admit_installed_basis(&installed).unwrap();
    assert!(matches!(
        port.export_recorded_runtime_world_baseline_in(&request, &admitted),
        Err(RuntimeWorldBaselineRefusal::BaselineCorrespondenceRequired)
    ));
    port.compare_current_exact(&admitted).unwrap();
    assert_eq!(
        bridge
            .rebuild_correspondence_allocation_index()
            .unwrap()
            .authoritative_semantic_dependencies(),
        1
    );
}

#[test]
fn authored_baseline_description_refuses_the_retired_empty_only_version() {
    let policy = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let request = worth_execution::ExecutionRequest::serial(&policy);
    let bytes = br#"{"version":1,"baseline":"bridge-runtime-world-uninstalled-baseline-v1","adapter_semantic_identity":"source","graph_role":"primary"}"#;
    assert!(matches!(
        RecordedRuntimeWorldBaseline::from_json_in(&request, bytes),
        Err(RuntimeWorldBaselineRefusal::UnsupportedVersion {
            observed: 1,
            supported: 2
        })
    ));
    let current = br#"{"version":2,"baseline":"bridge-runtime-world-baseline-v2","adapter_semantic_identity":"source","graph_role":"primary"}"#;
    let description = RecordedRuntimeWorldBaseline::from_json_in(&request, current).unwrap();
    let expected = br#"{"graph_role":"primary","adapter_semantic_identity":"source","baseline":"bridge-runtime-world-baseline-v2","version":2}"#;
    assert_eq!(
        description,
        RecordedRuntimeWorldBaseline::from_json_in(&request, expected).unwrap()
    );
}
