use crate::support::{clean_store, request};
use serde_json::{json, Value};
use worth_store_offline_integrity_observer::{
    compare_integrity_observations, encode_offline_integrity_report, observe_store,
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits as Limits,
};

fn reports() -> (Value, Value) {
    let fixture = clean_store("comparison-input");
    let report = observe_store(&request(&fixture)).unwrap();
    let offline: Value =
        serde_json::from_str(&encode_offline_integrity_report(&report).unwrap()).unwrap();
    // This is a protocol fixture, not evidence that a runtime validator agrees.
    let mut runtime = offline.clone();
    runtime["role"] = json!("runtime-integrity-observer");
    runtime["executable"] = json!("runtime-observer");
    runtime["process"] = json!("runtime-process");
    runtime["run"] = json!("runtime-independent-run");
    (runtime, offline)
}

#[test]
fn comparison_preserves_both_identities_and_exact_disagreements_without_a_winner() {
    let (runtime, mut offline) = reports();
    let clean = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(clean.counters().disagreements(), 0);
    offline["artifacts"][0]["outcome"] =
        json!({"posture":"indeterminate","reason":"source_changed"});
    let compared = compare_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        Limits::default(),
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
        Limits::default(),
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
            Limits::default()
        ),
        Err(Denial::InvalidObservation)
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
        Limits::default(),
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
        Limits::default(),
    )
    .unwrap();
    assert!(compared.encoded_report().contains("1..=11"));
}

#[test]
fn comparison_rejects_protocol_identity_role_scope_and_resource_errors() {
    let (runtime, offline) = reports();
    for (field, value, denial) in [
        ("version", json!(2), Denial::UnsupportedProtocol),
        (
            "role",
            json!("runtime-integrity-observer"),
            Denial::InvalidRole,
        ),
        ("store", json!("other-store"), Denial::StoreMismatch),
        (
            "scenario",
            json!("other-scenario"),
            Denial::ScenarioMismatch,
        ),
        ("process", runtime["process"].clone(), Denial::SameObserver),
        ("run", json!(""), Denial::InvalidIdentity),
        (
            "compatibility",
            json!({"earliest":2,"latest":2}),
            Denial::InvalidCompatibility,
        ),
    ] {
        let mut changed = offline.clone();
        changed[field] = value;
        assert_eq!(
            compare_integrity_observations(
                &runtime.to_string(),
                &changed.to_string(),
                Limits::default()
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
            Limits::default()
        )
        .unwrap_err(),
        Denial::DuplicateScope
    );
    for (limits, denial) in [
        (
            Limits::new(1, 100, 100_000).unwrap(),
            Denial::InputBoundExceeded,
        ),
        (
            Limits::new(100_000, 1, 100_000).unwrap(),
            Denial::ArtifactBoundExceeded,
        ),
        (
            Limits::new(100_000, 100, 1).unwrap(),
            Denial::ReportBoundExceeded,
        ),
    ] {
        assert_eq!(
            compare_integrity_observations(&runtime.to_string(), &offline.to_string(), limits)
                .unwrap_err(),
            denial
        );
    }
}

#[test]
fn real_compare_binary_preserves_inputs_and_rejects_alias_output_and_repair_flags() {
    let fixture = clean_store("comparison-binary");
    let (runtime, offline) = reports();
    let directory = fixture.store.parent().unwrap();
    let runtime_path = directory.join("runtime.json");
    let offline_path = directory.join("offline.json");
    let output_path = directory.join("comparison.json");
    std::fs::write(&runtime_path, runtime.to_string()).unwrap();
    std::fs::write(&offline_path, offline.to_string()).unwrap();
    let invoke = |output: &std::path::Path, extra: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
            .arg("compare")
            .arg("--runtime-observation")
            .arg(&runtime_path)
            .arg("--offline-observation")
            .arg(&offline_path)
            .arg("--report")
            .arg(output)
            .args(extra)
            .output()
            .unwrap()
    };
    let result = invoke(&output_path, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result: Value = serde_json::from_slice(&std::fs::read(&output_path).unwrap()).unwrap();
    assert_eq!(result["runtime"], runtime);
    assert!(!invoke(&runtime_path, &[]).status.success());
    let alias = directory.join("alias.json");
    std::fs::hard_link(&runtime_path, &alias).unwrap();
    assert!(!invoke(&alias, &[]).status.success());
    assert!(
        !invoke(&directory.join("repair.json"), &["--repair", "yes"])
            .status
            .success()
    );
    assert_eq!(
        std::fs::read_to_string(&runtime_path).unwrap(),
        runtime.to_string()
    );
    assert_eq!(
        std::fs::read_to_string(&offline_path).unwrap(),
        offline.to_string()
    );
}
