use std::collections::BTreeSet;

pub(super) fn assert_exact_request_lineage(
    evidence: &serde_json::Value,
    request: &serde_json::Value,
) {
    let attempt = request["attempt"].as_u64().expect("Query attempt identity");
    let binding = request["binding"].as_u64().expect("Query binding identity");
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
    let mounted_frame = text_work[0]["mounted_frame"].as_u64().unwrap();
    let host_lineage = text_work[0]["host_lineage"].as_u64().unwrap();
    assert_eq!(evidence["presentation"]["frame"], mounted_frame);
    assert_eq!(
        text_work[0]["active_mechanic_identity_digest"]
            .as_str()
            .map(str::len),
        Some(64)
    );
    let work_mechanics = text_work[0]["active_mechanics"]
        .as_array()
        .expect("owner-issued active mechanic identities")
        .iter()
        .map(mechanic_identity)
        .collect::<BTreeSet<_>>();
    assert!(
        !work_mechanics.is_empty(),
        "current request has no mounted text mechanics"
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
    // An empty admitted text mechanic needs no atlas pin. Every actual pin
    // must still name an active layout, and its exact inventory was joined above.
    assert!(pinned_layouts.is_subset(&active_layouts));

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
        row["surface"] == physical[0]["surface"]
            && row["host_surface"] == physical[0]["host_surface"]
            && row["host_session"] == host_lineage
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
