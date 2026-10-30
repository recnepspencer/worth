pub(crate) use crate::{
    store_offline_integrity_observer_support::request, support::fixtures::clean_store,
};
pub(crate) use serde_json::{json, Value};
pub(crate) use worth_store_offline_integrity_observer::{
    compare_integrity_observations, encode_offline_integrity_report, observe_store,
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits as Limits,
};

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
