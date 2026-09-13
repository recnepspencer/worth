use std::collections::BTreeSet;

pub(super) fn assert_exact_request_lineage(
    evidence: &serde_json::Value,
    request: &serde_json::Value,
) {
    let attempt = request["attempt"].as_u64().expect("Query attempt identity");
    let binding = request["binding"].as_u64().expect("Query binding identity");
    let subscribers = evidence["semantic_frontiers"]
        .as_array()
        .expect("semantic frontier evidence")
        .iter()
        .flat_map(|frontier| {
            assert!(frontier["source_deliveries"].as_u64().is_some());
            assert!(frontier["scope_rejections"].as_array().is_some());
            frontier["subscribers"]
                .as_array()
                .expect("owner-issued subscriber evidence")
        })
        .filter(|subscriber| {
            subscriber["attempt"].as_u64() == Some(attempt)
                && subscriber["binding"].as_u64() == Some(binding)
        })
        .collect::<Vec<_>>();
    assert!(
        !subscribers.is_empty(),
        "the current Query request has no exact semantic subscriber"
    );

    let request_axes = subscribers
        .iter()
        .map(|subscriber| {
            for digest in [
                "content_digest",
                "layout_digest",
                "foreground_digest",
                "raster_key_set_digest",
                "source_digest",
                "immediate_dependency_digest",
            ] {
                assert_eq!(subscriber[digest].as_str().map(str::len), Some(64));
                assert!(subscriber[digest]
                    .as_str()
                    .is_some_and(|value| value != "0".repeat(64)));
            }
            assert!(subscriber["mounted_instance"]
                .as_u64()
                .is_some_and(|value| value > 0));
            assert!(subscriber["semantic_slot"].as_u64().is_some());
            (
                subscriber["mounted_frame"].as_u64().unwrap(),
                subscriber["semantic_surface"].as_u64().unwrap(),
                subscriber["host_surface"].as_u64().unwrap(),
                subscriber["host_lineage"].as_u64().unwrap(),
            )
        })
        .collect::<BTreeSet<_>>();
    let request_axes = request_axes.into_iter().collect::<Vec<_>>();
    let [(mounted_frame, semantic_surface, host_surface, host_lineage)] = request_axes.as_slice()
    else {
        panic!("one Query request must retain one exact mounted/native lineage");
    };

    let text_work = evidence["text_presentation_work"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|work| work["attempt"] == attempt && work["binding"] == binding)
        .collect::<Vec<_>>();
    assert_eq!(
        text_work.len(),
        1,
        "request must have one exact text-work row"
    );
    assert_eq!(text_work[0]["mounted_frame"], *mounted_frame);
    assert_eq!(text_work[0]["host_lineage"], *host_lineage);
    assert_eq!(
        text_work[0]["active_mechanic_identity_digest"]
            .as_str()
            .map(str::len),
        Some(64)
    );
    let subscriber_mechanics = subscribers
        .iter()
        .filter(|subscriber| !subscriber["removal"].as_bool().unwrap())
        .map(|subscriber| mechanic_identity(subscriber))
        .collect::<BTreeSet<_>>();
    let work_mechanics = text_work[0]["active_mechanics"]
        .as_array()
        .expect("owner-issued active mechanic identities")
        .iter()
        .map(mechanic_identity)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        work_mechanics, subscriber_mechanics,
        "semantic subscribers and performed text work must retain the same exact mechanics"
    );

    let pin_frames = evidence["text_pin_frames"]
        .as_array()
        .expect("owner-issued pin-frame evidence");
    let current_pins = pin_frames
        .last()
        .and_then(serde_json::Value::as_array)
        .expect("the current request must retain one pin frame");
    assert_eq!(
        current_pins.len() as u64,
        text_work[0]["binding_pins"].as_u64().unwrap(),
        "the current pin inventory must match performed binding pins"
    );
    let pinned_identities = current_pins
        .iter()
        .map(pin_identity)
        .collect::<BTreeSet<_>>();
    let request_pin_identities = text_work[0]["binding_pin_identities"]
        .as_array()
        .expect("Query-basis binding pin identities")
        .iter()
        .map(pin_identity)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        pinned_identities, request_pin_identities,
        "native current pins must exactly match the Query-basis pin inventory"
    );
    let pinned_layouts = pinned_identities
        .iter()
        .map(|pin| pin.0.as_str())
        .collect::<BTreeSet<_>>();
    let active_layouts = work_mechanics
        .iter()
        .map(|mechanic| mechanic.3.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(pinned_layouts, active_layouts);

    let physical = evidence["physical_signal_transitions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["work"] == "Presentation" && row["attempt"] == attempt && row["binding"] == binding
        })
        .collect::<Vec<_>>();
    assert!(
        !physical.is_empty(),
        "request has no physical Signal evidence"
    );
    assert!(physical.iter().all(|row| {
        row["surface"] == *semantic_surface
            && row["host_surface"] == *host_surface
            && row["host_session"] == *host_lineage
            && row["request_sequence"]
                .as_u64()
                .is_some_and(|value| value > 0)
    }));
    assert!(
        physical.iter().any(|row| {
            row["origin"] == "NativeExternalPort"
                && row["external_status"] == "Completed"
                && row["settlement"] == "Completed"
        }),
        "the current request must retain one real native-port completion"
    );
}

fn mechanic_identity(row: &serde_json::Value) -> (u64, u64, Option<String>, String, String) {
    (
        row["mounted_instance"].as_u64().unwrap(),
        row["semantic_slot"].as_u64().unwrap(),
        row["collection_row"].as_str().map(str::to_owned),
        row["layout_digest"].as_str().unwrap().to_owned(),
        row["raster_key_set_digest"].as_str().unwrap().to_owned(),
    )
}

fn pin_identity(row: &serde_json::Value) -> (String, String) {
    (
        row["layout"].as_str().unwrap().to_owned(),
        row["raster_key"].as_str().unwrap().to_owned(),
    )
}
