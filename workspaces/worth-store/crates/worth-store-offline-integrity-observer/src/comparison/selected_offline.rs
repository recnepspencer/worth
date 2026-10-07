//! Version-two selected rows from the independent observer's admitted walk.
//! The version-one physical-path report remains byte-for-byte unchanged.

use serde_json::{json, Value};

use super::{
    disagreement::encode_bounded,
    selected_protocol::{PROTOCOL, VERSION},
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits,
};
use crate::{
    encode_offline_integrity_report, OfflineIntegrityReport, OfflineIntegrityReportCompleteness,
    OFFLINE_OBSERVER_ROLE_IDENTITY,
};

pub fn encode_offline_selected_integrity_observation(
    report: &OfflineIntegrityReport,
    limits: PhysicalIntegrityComparisonLimits,
) -> Result<String, Denial> {
    if report.completeness() != OfflineIntegrityReportCompleteness::Complete {
        return Err(Denial::IncompleteObservation);
    }
    let selected = report
        .selected_root()
        .ok_or(Denial::SelectedRootUnavailable)?;
    let store = report.store_identity().ok_or(Denial::StoreMismatch)?;
    let v1 = encode_offline_integrity_report(report).map_err(|_| Denial::ReportBoundExceeded)?;
    if v1.len() as u64 > limits.maximum_input_bytes() {
        return Err(Denial::InputBoundExceeded);
    }
    let v1: Value = serde_json::from_str(&v1).map_err(|_| Denial::MalformedProtocol)?;
    let mut artifacts = Vec::new();
    for row in v1["artifacts"]
        .as_array()
        .ok_or(Denial::MalformedProtocol)?
    {
        let Some(family) = row["family"].as_str() else {
            return Err(Denial::MalformedProtocol);
        };
        if super::selected_protocol::family(family).is_none() {
            continue;
        }
        let identity = row["identity"].as_str().ok_or(Denial::MalformedProtocol)?;
        if !report.selected_record(identity) {
            continue;
        }
        let record = selected_record(identity).ok_or(Denial::InvalidRecordIdentity)?;
        super::selected_protocol::record(&record)?;
        let path = row["path"].as_str().ok_or(Denial::MalformedProtocol)?;
        artifacts.push(json!({
            "family": family,
            "record": record,
            "outcome": row["outcome"],
            "physical_path": path,
        }));
        if artifacts.len() as u64 > limits.maximum_artifacts() {
            return Err(Denial::ArtifactBoundExceeded);
        }
    }
    let context = report.protocol_context();
    let document = json!({
        "protocol": PROTOCOL,
        "version": VERSION,
        "role": OFFLINE_OBSERVER_ROLE_IDENTITY,
        "executable": context.executable_identity(),
        "process": context.process_identity(),
        "run": context.run_identity(),
        "scenario": context.scenario_identity(),
        "store": store,
        "compatibility": {"earliest": VERSION, "latest": VERSION},
        "declared_limits": v1["declared_limits"],
        "consumed": v1["consumed"],
        "completeness": "complete",
        "selected_root": {"generation": selected.generation, "reference": selected.reference},
        "artifacts": artifacts,
    });
    let wire = encode_bounded(&document, limits.maximum_report_bytes())?;
    // The same strict parser governs emitted and external version-two input.
    super::selected_protocol::parse(&wire, OFFLINE_OBSERVER_ROLE_IDENTITY, limits)?;
    Ok(wire)
}

fn selected_record(identity: &str) -> Option<String> {
    if let Some(record) = identity.strip_prefix("blob-record:") {
        return Some(record.to_owned());
    }
    let suffix = identity.strip_prefix("index-record:")?;
    let (epoch, ordinal) = suffix.split_once(':')?;
    if epoch.len() != 32 || ordinal.len() != 16 {
        return None;
    }
    let ordinal = u64::from_str_radix(ordinal, 16).ok()?;
    let mut result = epoch.to_owned();
    for byte in ordinal.to_le_bytes() {
        use std::fmt::Write;
        write!(&mut result, "{byte:02x}").ok()?;
    }
    Some(result)
}
