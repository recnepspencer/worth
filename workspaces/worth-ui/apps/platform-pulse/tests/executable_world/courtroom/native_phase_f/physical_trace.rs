pub(super) fn assert_indeterminate(evidence: &serde_json::Value, request: &serde_json::Value) {
    let physical = transitions(evidence);
    let exact = |row: &serde_json::Value| {
        row["work"] == "Presentation"
            && row["attempt"] == request["attempt"]
            && row["binding"] == request["binding"]
    };
    let qualified = physical
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            exact(row)
                && row["origin"] == "QualifiedExternalPort"
                && row["external_status"] == "EffectsIndeterminate"
                && row["settlement"] == "Indeterminate"
        })
        .collect::<Vec<_>>();
    let [(qualified_index, qualified)] = qualified.as_slice() else {
        panic!("the exact Query request must receive one qualified indeterminate observation");
    };
    assert!(qualified["host_session"]
        .as_u64()
        .is_some_and(|value| value > 0));
    assert!(qualified["surface"].as_u64().is_some_and(|value| value > 0));
    assert!(qualified["host_surface"]
        .as_u64()
        .is_some_and(|value| value > 0));
    assert!(qualified["request_sequence"]
        .as_u64()
        .is_some_and(|value| value > 0));
    let native_pending = physical[..*qualified_index]
        .iter()
        .filter(|row| {
            exact(row)
                && row["origin"] == "NativeExternalPort"
                && row["external_status"] == "Pending"
                && row["settlement"] == "Pending"
                && row["host_session"] == qualified["host_session"]
                && row["surface"] == qualified["surface"]
                && row["host_surface"] == qualified["host_surface"]
                && row["request_sequence"] == qualified["request_sequence"]
        })
        .count();
    assert_eq!(native_pending, 1, "indeterminate qualification must follow one real native-port pending observation for the exact physical request");
}

pub(super) fn assert_duplicate_rejection(
    evidence: &serde_json::Value,
    duplicate: &serde_json::Value,
) {
    let physical = transitions(evidence);
    let matching = physical
        .iter()
        .enumerate()
        .filter(|row| {
            row.1["work"] == "Presentation"
                && row.1["attempt"] == duplicate["attempt"]
                && row.1["binding"] == duplicate["binding"]
                && row.1["origin"] == "QualifiedExternalPort"
                && row.1["external_status"] == "Completed"
                && row.1["settlement"] == "Stale"
        })
        .collect::<Vec<_>>();
    let [(duplicate_index, physical_duplicate)] = matching.as_slice() else {
        panic!("the exact duplicate Query rejection must follow one stale physical observation");
    };
    for field in [
        "host_session",
        "surface",
        "host_surface",
        "request_sequence",
    ] {
        assert!(physical_duplicate[field]
            .as_u64()
            .is_some_and(|value| value > 0));
    }
    let original = physical[..*duplicate_index]
        .iter()
        .filter(|row| {
            row["work"] == "Presentation"
                && row["attempt"] == duplicate["attempt"]
                && row["binding"] == duplicate["binding"]
                && row["origin"] == "NativeExternalPort"
                && row["external_status"] == "Completed"
                && row["settlement"] == "Completed"
                && row["host_session"] == physical_duplicate["host_session"]
                && row["surface"] == physical_duplicate["surface"]
                && row["host_surface"] == physical_duplicate["host_surface"]
                && row["request_sequence"] == physical_duplicate["request_sequence"]
        })
        .count();
    assert_eq!(
        original, 1,
        "duplicate rejection must follow the exact accepted native completion"
    );
}

pub(super) fn assert_supersession(
    evidence: &serde_json::Value,
    predecessor: &serde_json::Value,
    deferred: &serde_json::Value,
    successor: &serde_json::Value,
) {
    let physical = transitions(evidence);
    let matches = |row: &serde_json::Value, request: &serde_json::Value, settlement: &str| {
        row["work"] == "Presentation"
            && row["attempt"] == request["attempt"]
            && row["binding"] == request["binding"]
            && row["origin"] == "NativeExternalPort"
            && row["external_status"] == "Completed"
            && row["settlement"] == settlement
    };
    assert!(
        physical
            .iter()
            .any(|row| matches(row, predecessor, "Completed")),
        "the semantically superseded predecessor physically completes while its successor awaits the atlas"
    );
    let work = evidence["text_presentation_work"].as_array().unwrap();
    let exact_work = |request: &serde_json::Value| {
        let rows = work
            .iter()
            .filter(|row| {
                row["attempt"] == request["attempt"] && row["binding"] == request["binding"]
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        rows[0]
    };
    let before = exact_work(deferred);
    let retry = exact_work(successor);
    for field in [
        "mounted_frame",
        "active_mechanics",
        "binding_pin_identities",
        "glyph_run_transcript_digest",
        "intrinsic_glyph_transcript_digest",
    ] {
        assert_eq!(
            before[field], retry[field],
            "atlas retry preserves exact {field}"
        );
    }
    let atlas = physical
        .iter()
        .position(|row| {
            row["work"] == "AtlasUpload"
                && row["attempt"] == deferred["attempt"]
                && row["binding"] == deferred["binding"]
                && row["origin"] == "NativeExternalPort"
                && row["settlement"] == "Completed"
        })
        .unwrap();
    let first = physical
        .iter()
        .position(|row| matches(row, predecessor, "Completed"))
        .unwrap();
    let last = physical
        .iter()
        .position(|row| matches(row, successor, "Completed"))
        .unwrap();
    assert!(
        first < atlas && atlas < last,
        "A completes, B's atlas settles, then exact B retry paints"
    );
    assert!(
        physical
            .iter()
            .any(|row| matches(row, successor, "Completed")),
        "the exact successor completion must remain physically current"
    );
}

fn transitions(evidence: &serde_json::Value) -> &[serde_json::Value] {
    evidence["physical_signal_transitions"]
        .as_array()
        .expect("the product retains physical Signal transition observations")
}
