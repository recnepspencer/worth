use super::CommittedObservationEventSummary;
use crate::data::resource::ResourceObservationEvent;
use crate::logic::transaction::ObservationPolicy;

fn current_event() -> serde_json::Value {
    serde_json::json!({
        "observer_id":1,
        "handle_id":2,
        "policy":{
            "schema_version":"worth.signal.observation-policy.v2",
            "trigger":"Visited",
            "delivery_mode":"PerCommittedTransaction"
        },
        "outcome":"Delivered",
        "visited":true,
        "recomputed":false,
        "meaningful_change":false,
        "trigger_matched":true
    })
}

fn renamed_to_touched(mut event: serde_json::Value) -> serde_json::Value {
    let fields = event.as_object_mut().unwrap();
    let visited = fields.remove("visited").unwrap();
    fields.insert("touched".to_owned(), visited);
    event
}

#[test]
fn committed_and_resource_events_use_only_the_visited_spelling() {
    let mut ordinary = current_event();
    ordinary["observed_nodes"] = serde_json::json!({"nodes":[]});
    ordinary["matched_nodes"] = serde_json::json!({"nodes":[]});
    assert!(
        serde_json::from_value::<CommittedObservationEventSummary>(renamed_to_touched(
            ordinary.clone()
        ))
        .is_err()
    );
    let ordinary: CommittedObservationEventSummary = serde_json::from_value(ordinary).unwrap();
    assert!(ordinary.visited);
    assert_eq!(ordinary.policy, ObservationPolicy::visited());
    assert_visited_vocabulary(serde_json::to_value(ordinary).unwrap());

    let mut resource = current_event();
    resource["matched_resource_nodes"] = serde_json::json!([]);
    assert!(
        serde_json::from_value::<ResourceObservationEvent>(renamed_to_touched(resource.clone()))
            .is_err()
    );
    let resource: ResourceObservationEvent = serde_json::from_value(resource).unwrap();
    assert!(resource.visited());
    assert_eq!(resource.policy(), ObservationPolicy::visited());
    assert_visited_vocabulary(serde_json::to_value(resource).unwrap());
}

fn assert_visited_vocabulary(wire: serde_json::Value) {
    assert_eq!(wire["visited"], true);
    assert!(wire.get("touched").is_none());
    assert_eq!(wire["policy"]["trigger"], "Visited");
    assert_eq!(
        wire["policy"]["schema_version"],
        "worth.signal.observation-policy.v2"
    );
}
