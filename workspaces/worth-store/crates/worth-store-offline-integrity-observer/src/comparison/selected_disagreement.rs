//! Selected C.11 comparison joins on logical RecordId, never physical path.

use std::collections::BTreeMap;

use serde_json::json;
use worth_foundational::{PhysicalArtifactIdentity, PhysicalIntegrityDisagreement};

use super::{
    disagreement::encode_bounded,
    selected_protocol::{family, parse, SelectedArtifact, RUNTIME_ROLE},
    PhysicalIntegrityComparison, PhysicalIntegrityComparisonCounters,
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits,
};

/// Compare independently produced selected-record observations without
/// selecting a winner or treating offline placement paths as join authority.
pub fn compare_selected_integrity_observations(
    runtime: &str,
    offline: &str,
    limits: PhysicalIntegrityComparisonLimits,
) -> Result<PhysicalIntegrityComparison, Denial> {
    let runtime_report = parse(runtime, RUNTIME_ROLE, limits)?;
    let offline_report = parse(offline, crate::OFFLINE_OBSERVER_ROLE_IDENTITY, limits)?;
    if runtime_report.store != offline_report.store {
        return Err(Denial::StoreMismatch);
    }
    if runtime_report.scenario != offline_report.scenario {
        return Err(Denial::ScenarioMismatch);
    }
    if runtime_report.selected_root != offline_report.selected_root {
        return Err(Denial::SelectedRootMismatch);
    }
    if runtime_report.executable == offline_report.executable
        || runtime_report.process == offline_report.process
        || runtime_report.run == offline_report.run
    {
        return Err(Denial::SameObserver);
    }
    let runtime_rows = index(&runtime_report.artifacts)?;
    let offline_rows = index(&offline_report.artifacts)?;
    if runtime_rows.is_empty() {
        return Err(Denial::InvalidObservation);
    }
    let mut comparisons = Vec::with_capacity(runtime_rows.len());
    let mut agreements = 0_u64;
    let mut disagreements = 0_u64;
    for record in runtime_rows.keys() {
        let runtime_row = runtime_rows.get(record).copied();
        let offline_row = offline_rows.get(record).copied();
        let fields = differing_fields(runtime_row, offline_row);
        let agreement = fields.is_empty();
        if agreement {
            agreements += 1;
        } else {
            disagreements += 1;
        }
        let record_hex = runtime_row
            .expect("declared runtime target")
            .record
            .as_str();
        let posture_disagreement = match (runtime_row, offline_row) {
            (Some(runtime_row), Some(offline_row)) => {
                family(&runtime_row.family).and_then(|family| {
                    PhysicalIntegrityDisagreement::new(
                        family,
                        PhysicalArtifactIdentity::new(record_hex.to_owned()).ok()?,
                        runtime_row.outcome.posture(),
                        offline_row.outcome.posture(),
                    )
                })
            }
            _ => None,
        };
        comparisons.push(json!({
            "record": record_hex,
            "runtime_family": runtime_row.map(|row| row.family.as_str()),
            "offline_family": offline_row.map(|row| row.family.as_str()),
            "agreement": agreement,
            "different_fields": fields,
            "posture_disagreement": posture_disagreement,
        }));
    }
    let offline_unobserved: Vec<_> = offline_rows
        .iter()
        .filter(|(record, _)| !runtime_rows.contains_key(record))
        .map(|(_, row)| json!({"record": row.record, "family": row.family}))
        .collect();
    let offline_unobserved_count = offline_unobserved.len();
    let document = json!({
        "protocol": "store.physical.integrity-comparison",
        "version": 2,
        "mode": "selected_record",
        "comparison_scope": "runtime_declared_targets",
        "selected_root": runtime_report.selected_root,
        "runtime": runtime_report,
        "offline": offline_report,
        "comparisons": comparisons,
        "offline_unobserved": offline_unobserved,
        "consumed": {
            "agreements": agreements,
            "disagreements": disagreements,
            "runtime_targets": runtime_rows.len(),
            "offline_unobserved": offline_unobserved_count,
            "input_bytes": runtime.len() + offline.len(),
        },
    });
    let wire = encode_bounded(&document, limits.maximum_report_bytes())?;
    Ok(PhysicalIntegrityComparison {
        counters: PhysicalIntegrityComparisonCounters {
            agreements,
            disagreements,
            input_bytes: (runtime.len() + offline.len()) as u64,
            report_bytes: wire.len() as u64,
        },
        wire,
    })
}

type Record = super::selected_protocol::SelectedRecordIdentity;

fn index(artifacts: &[SelectedArtifact]) -> Result<BTreeMap<Record, &SelectedArtifact>, Denial> {
    let mut indexed = BTreeMap::new();
    for artifact in artifacts {
        let record = super::selected_protocol::record(&artifact.record)?;
        if indexed.insert(record, artifact).is_some() {
            return Err(Denial::DuplicateScope);
        }
    }
    Ok(indexed)
}

fn differing_fields(
    runtime: Option<&SelectedArtifact>,
    offline: Option<&SelectedArtifact>,
) -> Vec<&'static str> {
    let (Some(runtime), Some(offline)) = (runtime, offline) else {
        return vec!["presence"];
    };
    let mut fields = Vec::new();
    if runtime.family != offline.family {
        fields.push("family");
    }
    if runtime.outcome.posture() != offline.outcome.posture() {
        fields.push("disposition");
    }
    if runtime.outcome != offline.outcome {
        fields.push("outcome");
    }
    fields
}
