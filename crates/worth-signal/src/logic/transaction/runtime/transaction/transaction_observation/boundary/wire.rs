use super::CommittedObservationEventSummary;
use crate::data::resource::ResourceObservationEvent;
use crate::logic::transaction::ObservationPolicy;

fn legacy_event() -> serde_json::Value {
    serde_json::json!({
        "observer_id":1,
        "handle_id":2,
        "policy":{"trigger":"Touched","delivery_mode":"PerCommittedTransaction"},
        "outcome":"Delivered",
        "touched":true,
        "recomputed":false,
        "meaningful_change":false,
        "trigger_matched":true
    })
}

#[test]
fn legacy_committed_and_resource_events_emit_visited_public_fields() {
    let mut ordinary = legacy_event();
    ordinary["observed_nodes"] = serde_json::json!({"nodes":[]});
    ordinary["matched_nodes"] = serde_json::json!({"nodes":[]});
    let ordinary: CommittedObservationEventSummary = serde_json::from_value(ordinary).unwrap();
    assert!(ordinary.visited);
    assert_eq!(ordinary.policy, ObservationPolicy::visited());
    assert_new_vocabulary(serde_json::to_value(ordinary).unwrap());

    let mut resource = legacy_event();
    resource["matched_resource_nodes"] = serde_json::json!([]);
    let resource: ResourceObservationEvent = serde_json::from_value(resource).unwrap();
    assert!(resource.visited());
    assert_eq!(resource.policy(), ObservationPolicy::visited());
    assert_new_vocabulary(serde_json::to_value(resource).unwrap());
}

fn assert_new_vocabulary(wire: serde_json::Value) {
    assert_eq!(wire["visited"], true);
    assert!(wire.get("touched").is_none());
    assert_eq!(wire["policy"]["trigger"], "Visited");
    assert_eq!(
        wire["policy"]["schema_version"],
        "worth.signal.observation-policy.v2"
    );
}
