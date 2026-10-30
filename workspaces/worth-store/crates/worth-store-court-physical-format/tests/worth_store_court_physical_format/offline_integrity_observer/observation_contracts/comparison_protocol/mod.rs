#[allow(unused_imports)]
use crate::comparison_protocol::*;

#[test]
fn comparison_preserves_both_identities_and_exact_disagreements_without_a_winner() {
    let (runtime, mut offline) = reports();
    let clean = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default(),
    )
    .unwrap();
    assert_eq!(clean.counters().disagreements(), 0);
    offline["artifacts"][0]["outcome"] =
        json!({"posture":"indeterminate","reason":"source_changed"});
    let compared = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default(),
    )
    .unwrap();
    assert_eq!(compared.counters().disagreements(), 1);
    let wire: Value = serde_json::from_str(compared.encoded_report()).unwrap();
    assert_eq!(wire["runtime"], runtime);
    assert_eq!(wire["offline"], offline);
    assert_eq!(wire["version"], 1);
    assert!(wire.get("verdict").is_none());
    let differences: Vec<_> = wire["comparisons"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["agreement"] == false)
        .collect();
    assert_eq!(differences[0]["different_fields"], json!(["outcome"]));
    assert!(!differences[0]["posture_disagreement"].is_null());
}

#[test]
fn runtime_partial_source_range_is_preserved_and_validated_in_version_one() {
    let (mut runtime, offline) = reports();
    runtime["artifacts"][0]["outcome"] = json!({"posture":"indeterminate","reason":"stable_range_not_proven","observed_range":{"offset":0,"length":16}});
    let compared = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default(),
    )
    .unwrap();
    let wire: Value = serde_json::from_str(compared.encoded_report()).unwrap();
    assert_eq!(
        wire["runtime"]["artifacts"][0]["outcome"]["observed_range"]["length"],
        16
    );
    runtime["artifacts"][0]["outcome"]["observed_range"]["length"] = json!(0);
    assert_eq!(
        compare_integrity_observations(
            &runtime.to_string(),
            &offline.to_string(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default()
        ),
        Err(worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidObservation)
    );
}

#[test]
fn comparison_does_not_flatten_same_posture_localization_or_unsupported_windows() {
    let (mut runtime, mut offline) = reports();
    let damaged = json!({"posture":"damaged","cause":"checksum_mismatch","damaged_range":{"offset":44,"length":4},"field":"frame_checksum","blast_radius":"artifact"});
    runtime["artifacts"][0]["outcome"] = damaged.clone();
    offline["artifacts"][0]["outcome"] = damaged;
    offline["artifacts"][0]["outcome"]["damaged_range"]["offset"] = json!(48);
    let compared = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default(),
    )
    .unwrap();
    assert_eq!(compared.counters().disagreements(), 1);
    let encoded: Value = serde_json::from_str(compared.encoded_report()).unwrap();
    assert_eq!(
        encoded["offline"]["artifacts"][0]["outcome"]["damaged_range"]["offset"],
        48
    );
    offline["artifacts"][0]["outcome"] = json!({"posture":"unsupported","axis":"frame_schema","observed":12,"supported":"1..=11","range":{"offset":8,"length":2}});
    let compared = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default(),
    )
    .unwrap();
    assert!(compared.encoded_report().contains("1..=11"));
}

#[test]
fn comparison_rejects_protocol_identity_role_scope_and_resource_errors() {
    let (runtime, offline) = reports();
    for (field, value, denial) in [
        ("version", json!(2), worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::UnsupportedProtocol),
        (
            "role",
            json!("runtime-integrity-observer"),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidRole,
        ),
        ("store", json!("other-store"), worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::StoreMismatch),
        (
            "scenario",
            json!("other-scenario"),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::ScenarioMismatch,
        ),
        ("process", runtime["process"].clone(), worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::SameObserver),
        ("run", json!(""), worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidIdentity),
        (
            "compatibility",
            json!({"earliest":2,"latest":2}),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidCompatibility,
        ),
    ] {
        let mut changed = offline.clone();
        changed[field] = value;
        assert_eq!(
            compare_integrity_observations(
                &runtime.to_string(),
                &changed.to_string(),
                worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default()
            )
            .unwrap_err(),
            denial,
            "{field}"
        );
    }
    let mut duplicated = offline.clone();
    duplicated["artifacts"]
        .as_array_mut()
        .unwrap()
        .push(offline["artifacts"][0].clone());
    assert_eq!(
        compare_integrity_observations(
            &runtime.to_string(),
            &duplicated.to_string(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default()
        )
        .unwrap_err(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::DuplicateScope
    );
    for (limits, denial) in [
        (
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::new(1, 100, 100_000).unwrap(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InputBoundExceeded,
        ),
        (
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::new(100_000, 1, 100_000).unwrap(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::ArtifactBoundExceeded,
        ),
        (
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::new(100_000, 100, 1).unwrap(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::ReportBoundExceeded,
        ),
    ] {
        assert_eq!(
            compare_integrity_observations(&runtime.to_string(), &offline.to_string(), limits)
                .unwrap_err(),
            denial
        );
    }
}
