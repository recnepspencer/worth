use serde_json::{json, Value};
use worth_store_offline_integrity_observer::{
    compare_selected_integrity_observations, encode_offline_integrity_report,
    encode_offline_selected_integrity_observation, observe_store,
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits as Limits,
};

use crate::support::{clean_store, request};

const RECORD: &str = "222222222222222222222222222222220100000000000000";

fn selected_reports() -> (Value, Value) {
    let runtime = json!({
        "protocol": "store.physical.selected-integrity-observation",
        "version": 2,
        "role": "runtime-selected-scrub",
        "executable": "store-runtime",
        "process": "runtime-process",
        "run": "runtime-run",
        "scenario": "selected-chunk-journey",
        "store": "11111111111111111111111111111111",
        "compatibility": {"earliest": 2, "latest": 2},
        "declared_limits": {"targets": 1, "report_bytes": 8192},
        "consumed": {"targets": 1, "bytes": 64},
        "completeness": "complete",
        "selected_root": {"generation": 9, "reference": 9},
        "artifacts": [{"family":"blob_chunk_frame", "record":RECORD,
                       "outcome":{"posture":"intact"}}],
    });
    let mut offline = runtime.clone();
    offline["role"] = json!("offline-root-observer");
    offline["executable"] = json!("physical-store-integrity-observer");
    offline["process"] = json!("offline-process");
    offline["run"] = json!("offline-run");
    offline["artifacts"][0]["physical_path"] = json!("families/records/arenas/arena-1.data");
    (runtime, offline)
}

fn compare(runtime: &Value, offline: &Value) -> Result<Value, Denial> {
    let comparison = compare_selected_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        Limits::default(),
    )?;
    Ok(serde_json::from_str(comparison.encoded_report()).unwrap())
}

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
            Denial::StoreMismatch,
        ),
        (
            "scenario",
            json!("other-scenario"),
            Denial::ScenarioMismatch,
        ),
        (
            "selected_root",
            json!({"generation":10,"reference":10}),
            Denial::SelectedRootMismatch,
        ),
        ("role", json!("runtime-selected-scrub"), Denial::InvalidRole),
        (
            "executable",
            runtime["executable"].clone(),
            Denial::SameObserver,
        ),
        ("version", json!(1), Denial::UnsupportedProtocol),
        (
            "completeness",
            json!("indeterminate"),
            Denial::IncompleteObservation,
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
        Denial::InvalidRecordIdentity
    );
    changed = offline.clone();
    changed["artifacts"]
        .as_array_mut()
        .unwrap()
        .push(offline["artifacts"][0].clone());
    changed["artifacts"][1]["family"] = json!("blob_tree_node");
    assert_eq!(
        compare(&runtime, &changed).unwrap_err(),
        Denial::DuplicateScope
    );
    changed = runtime.clone();
    changed["artifacts"][0]["physical_path"] = json!("fabricated/path");
    assert_eq!(
        compare(&changed, &offline).unwrap_err(),
        Denial::InvalidObservation
    );
    assert_eq!(
        compare_selected_integrity_observations(
            &runtime.to_string(),
            &offline.to_string(),
            Limits::new(1, 100, 100_000).unwrap()
        )
        .unwrap_err(),
        Denial::InputBoundExceeded
    );
    assert_eq!(
        compare_selected_integrity_observations(
            &runtime.to_string(),
            &offline.to_string(),
            Limits::new(100_000, 1, 1).unwrap()
        )
        .unwrap_err(),
        Denial::ReportBoundExceeded
    );
}

#[test]
fn offline_v2_encoder_uses_admitted_root_without_changing_v1_wire() {
    let fixture = clean_store("selected-comparison-offline");
    let report = observe_store(&request(&fixture)).unwrap();
    let v1 = encode_offline_integrity_report(&report).unwrap();
    let selected =
        encode_offline_selected_integrity_observation(&report, Limits::default()).unwrap();
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
        std::process::Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
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
                crate::support::FIXTURE_OPEN_FILE_ARGUMENT,
                "--max-depth",
                "8",
                "--max-symlinks",
                "0",
                "--max-elapsed-ms",
                crate::support::FIXTURE_ELAPSED_ARGUMENT,
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
