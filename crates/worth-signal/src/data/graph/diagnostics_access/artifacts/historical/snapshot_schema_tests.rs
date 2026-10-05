use super::*;
use crate::data::node::NodeContract;

#[test]
fn schema_three_preserves_explicit_checked_result_limits_through_snapshot_restore() {
    for maximum in [0, 97] {
        let mut graph = SignalGraph::new();
        let node = graph
            .node()
            .with_contract(NodeContract::wildcard().with_max_checked_result_heap_bytes(maximum))
            .build();
        let snapshot = graph.capture_snapshot();
        assert_eq!(
            snapshot.meta.schema_version,
            SignalSnapshotMeta::SCHEMA_VERSION
        );
        let wire = serde_json::to_vec(&snapshot).unwrap();
        let decoded: SignalSnapshotV1 = serde_json::from_slice(&wire).unwrap();
        graph.restore_snapshot(&decoded).unwrap();
        assert_eq!(
            graph
                .get_contract(node)
                .unwrap()
                .execution
                .max_checked_result_heap_bytes,
            Some(maximum)
        );
    }
}

#[test]
fn schema_two_missing_limit_restores_without_changing_its_provenance() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let snapshot = graph.capture_snapshot();
    let mut null_wire = serde_json::to_value(&snapshot).unwrap();
    null_wire["meta"]["schema_version"] = serde_json::json!(2);
    let null_legacy: SignalSnapshotV1 = serde_json::from_value(null_wire).unwrap();
    graph.restore_snapshot(&null_legacy).unwrap();
    assert_eq!(null_legacy.meta.schema_version, 2);

    let mut wire = serde_json::to_value(&snapshot).unwrap();
    wire["meta"]["schema_version"] = serde_json::json!(2);
    remove_checked_result_limit(&mut wire);
    let legacy: SignalSnapshotV1 = serde_json::from_value(wire).unwrap();
    assert_eq!(legacy.meta.schema_version, 2);
    graph.restore_snapshot(&legacy).unwrap();
    assert_eq!(legacy.meta.schema_version, 2);
    assert_eq!(
        graph
            .get_contract(node)
            .unwrap()
            .execution
            .max_checked_result_heap_bytes,
        None
    );
    assert_eq!(graph.capture_snapshot().meta.schema_version, 3);
}

#[test]
fn schema_two_claiming_a_new_limit_is_rejected_before_restore() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_max_checked_result_heap_bytes(0))
        .build();
    let snapshot = graph.capture_snapshot();
    let replay_before = graph.replay_events().len();
    // Protect each serialized carrier independently: a legacy claim must fail
    // even when its new limit appears only in authority or only in diagnostics.
    for cleared_carrier in ["checkpoint_image", "diagnostic_graph"] {
        let mut wire = serde_json::to_value(&snapshot).unwrap();
        wire["meta"]["schema_version"] = serde_json::json!(2);
        remove_checked_result_limit(&mut wire[cleared_carrier]);
        let legacy: SignalSnapshotV1 = serde_json::from_value(wire).unwrap();
        let denial = graph.restore_snapshot(&legacy).unwrap_err();
        assert!(matches!(denial, SignalError::IncompatibleSnapshot { .. }));
        assert_eq!(graph.replay_events().len(), replay_before);
    }
    assert_eq!(
        graph
            .get_contract(node)
            .unwrap()
            .execution
            .max_checked_result_heap_bytes,
        Some(0)
    );
}

fn remove_checked_result_limit(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            fields.remove("max_checked_result_heap_bytes");
            for field in fields.values_mut() {
                remove_checked_result_limit(field);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                remove_checked_result_limit(item);
            }
        }
        _ => {}
    }
}
