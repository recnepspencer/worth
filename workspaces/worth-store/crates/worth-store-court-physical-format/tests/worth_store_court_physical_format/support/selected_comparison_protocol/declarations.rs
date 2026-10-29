pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {worth_store_offline_integrity_observer::compare_selected_integrity_observations, worth_store_offline_integrity_observer::encode_offline_integrity_report, worth_store_offline_integrity_observer::encode_offline_selected_integrity_observation, worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial as Denial, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits as Limits};

pub(crate) use {crate::support::fixtures::clean_store, crate::store_offline_integrity_observer_support::request};

pub(crate) const RECORD: &str = "222222222222222222222222222222220100000000000000";

pub(crate) fn selected_reports() -> (Value, Value) {
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

pub(crate) fn compare(runtime: &Value, offline: &Value) -> Result<Value, Denial> {
    let comparison = compare_selected_integrity_observations(
        &runtime.to_string(),
        &offline.to_string(),
        worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits::default(),
    )?;
    Ok(serde_json::from_str(comparison.encoded_report()).unwrap())
}