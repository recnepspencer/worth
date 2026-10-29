pub(crate) use {crate::support::fixtures::clean_store, crate::store_offline_integrity_observer_support::request};
pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {worth_store_offline_integrity_observer::compare_integrity_observations, worth_store_offline_integrity_observer::encode_offline_integrity_report, worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonDenial as Denial, worth_store_offline_integrity_observer::PhysicalIntegrityComparisonLimits as Limits};

pub(crate) fn reports() -> (Value, Value) {
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