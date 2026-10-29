#[allow(unused_imports)] use crate::selected_comparison_protocol::*;
use {serde_json::json, serde_json::Value};
use {worth_store_offline_integrity_observer::compare_selected_integrity_observations, worth_store_offline_integrity_observer::encode_offline_integrity_report, worth_store_offline_integrity_observer::encode_offline_selected_integrity_observation, worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial as Denial, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits as Limits};

use {crate::support::fixtures::clean_store, crate::store_offline_integrity_observer_support::request};

#[test]
fn selected_join_ignores_physical_path_but_retains_it_and_both_outcomes() {
    let (runtime, mut offline) = selected_reports();
    let clean = compare(&runtime, &offline).unwrap();
    assert_eq!(clean["version"], 2);
    assert_eq!(clean["comparisons"][0]["agreement"], true);
    assert_eq!(clean["offline"], offline);
    assert!(clean.get("verdict").is_none());
    offline["artifacts"][0]["physical_path"] = json!("other/physical/location.data");
    assert_eq!(
        compare(&runtime, &offline).unwrap()["comparisons"][0]["agreement"],
        true
    );

    offline["artifacts"][0]["outcome"] = json!({
        "posture":"damaged", "cause":"checksum_mismatch",
        "damaged_range":{"offset":44,"length":4},
        "field":"frame_checksum", "blast_radius":"canonical_frame"
    });
    let changed = compare(&runtime, &offline).unwrap();
    assert_eq!(
        changed["comparisons"][0]["different_fields"],
        json!(["disposition", "outcome"])
    );
    assert_eq!(changed["runtime"], runtime);
    assert_eq!(changed["offline"], offline);
    offline["artifacts"][0]["outcome"]["damaged_range"]["offset"] = json!(48);
    let mut runtime_damaged = runtime.clone();
    runtime_damaged["artifacts"][0]["outcome"] =
        changed["offline"]["artifacts"][0]["outcome"].clone();
    assert_eq!(
        compare(&runtime_damaged, &offline).unwrap()["comparisons"][0]["different_fields"],
        json!(["outcome"])
    );
}

#[test]
fn selected_join_reports_family_and_presence_without_fabricating_a_winner() {
    let (runtime, mut offline) = selected_reports();
    offline["artifacts"][0]["family"] = json!("blob_tree_node");
    let changed = compare(&runtime, &offline).unwrap();
    assert_eq!(
        changed["comparisons"][0]["different_fields"],
        json!(["family"])
    );
    assert_eq!(
        changed["comparisons"][0]["runtime_family"],
        "blob_chunk_frame"
    );
    assert_eq!(
        changed["comparisons"][0]["offline_family"],
        "blob_tree_node"
    );
    offline["artifacts"] = json!([]);
    assert_eq!(
        compare(&runtime, &offline).unwrap()["comparisons"][0]["different_fields"],
        json!(["presence"])
    );
}

#[test]
fn offline_selected_records_outside_runtime_declared_scope_are_explicitly_unobserved() {
    let (runtime, mut offline) = selected_reports();
    offline["artifacts"].as_array_mut().unwrap().push(json!({
        "family": "blob_tree_node",
        "record": "333333333333333333333333333333330200000000000000",
        "outcome": {"posture":"intact"},
        "physical_path": "families/records/arenas/older.data"
    }));
    let compared = compare(&runtime, &offline).unwrap();
    assert_eq!(compared["consumed"]["agreements"], 1);
    assert_eq!(compared["consumed"]["disagreements"], 0);
    assert_eq!(compared["consumed"]["offline_unobserved"], 1);
    assert_eq!(
        compared["offline_unobserved"][0]["family"],
        "blob_tree_node"
    );
    assert_eq!(compared["comparison_scope"], "runtime_declared_targets");
}

#[test]
fn selected_v2_rejects_wrong_basis_roles_ids_duplicates_and_bounds() {
    let (runtime, offline) = selected_reports();
    for (field, value, expected) in [
        (
            "store",
            json!("33333333333333333333333333333333"),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::StoreMismatch,
        ),
        (
            "scenario",
            json!("other-scenario"),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::ScenarioMismatch,
        ),
        (
            "selected_root",
            json!({"generation":10,"reference":10}),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::SelectedRootMismatch,
        ),
        ("role", json!("runtime-selected-scrub"), worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidRole),
        (
            "executable",
            runtime["executable"].clone(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::SameObserver,
        ),
        ("version", json!(1), worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::UnsupportedProtocol),
        (
            "completeness",
            json!("indeterminate"),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::IncompleteObservation,
        ),
    ] {
        let mut changed = offline.clone();
        changed[field] = value;
        assert_eq!(
            compare(&runtime, &changed).unwrap_err(),
            expected,
            "{field}"
        );
    }
    let mut changed = offline.clone();
    changed["artifacts"][0]["record"] = json!(format!("A{}", &RECORD[1..]));
    assert_eq!(
        compare(&runtime, &changed).unwrap_err(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidRecordIdentity
    );
    changed = offline.clone();
    changed["artifacts"]
        .as_array_mut()
        .unwrap()
        .push(offline["artifacts"][0].clone());
    changed["artifacts"][1]["family"] = json!("blob_tree_node");
    assert_eq!(
        compare(&runtime, &changed).unwrap_err(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::DuplicateScope
    );
    changed = runtime.clone();
    changed["artifacts"][0]["physical_path"] = json!("fabricated/path");
    assert_eq!(
        compare(&changed, &offline).unwrap_err(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InvalidObservation
    );
    assert_eq!(
        compare_selected_integrity_observations(
            &runtime.to_string(),
            &offline.to_string(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::new(1, 100, 100_000).unwrap()
        )
        .unwrap_err(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::InputBoundExceeded
    );
    assert_eq!(
        compare_selected_integrity_observations(
            &runtime.to_string(),
            &offline.to_string(),
            worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::new(100_000, 1, 1).unwrap()
        )
        .unwrap_err(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial::ReportBoundExceeded
    );
}

#[test]
fn offline_v2_encoder_uses_admitted_root_without_changing_v1_wire() {
    let fixture = clean_store("selected-comparison-offline");
    let report = observe_store(&request(&fixture)).unwrap();
    let v1 = encode_offline_integrity_report(&report).unwrap();
    let selected =
        encode_offline_selected_integrity_observation(&report, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default()).unwrap();
    let wire: Value = serde_json::from_str(&selected).unwrap();
    assert_eq!(wire["version"], 2);
    assert_eq!(wire["role"], "offline-root-observer");
    assert_eq!(
        wire["selected_root"]["generation"],
        wire["selected_root"]["reference"]
    );
    assert_eq!(serde_json::from_str::<Value>(&v1).unwrap()["version"], 1);
}

#[test]
fn observe_selected_cli_emits_from_a_distinct_real_process() {
    let fixture = clean_store("selected-comparison-child");
    let output =
        std::process::Command::new(worth_store_testkit_executables::physical_store_integrity_observer())
            .args([
                "observe-selected",
                "--store-root",
                fixture.store.to_str().unwrap(),
                "--report",
                fixture.report.to_str().unwrap(),
                "--max-entries",
                "100",
                "--max-bytes",
                "16384",
                "--max-open-files",
                crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_ARGUMENT,
                "--max-depth",
                "8",
                "--max-symlinks",
                "0",
                "--max-elapsed-ms",
                crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_ARGUMENT,
                "--max-report-bytes",
                "65536",
                "--run",
                "child-run",
                "--scenario",
                "child-selected-scenario",
            ])
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&std::fs::read(&fixture.report).unwrap()).unwrap();
    assert_eq!(report["version"], 2);
    assert_eq!(report["role"], "offline-root-observer");
    assert_eq!(report["executable"], "physical_store_integrity_observer");
    assert_ne!(report["process"], std::process::id().to_string());
    assert_eq!(report["scenario"], "child-selected-scenario");
}
