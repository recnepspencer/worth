use super::{artifact_edit::ArtifactOperator as Operator, artifact_inventory::ArtifactGranule};
use serde_json::Value;

mod journal;

pub(super) fn require_aggregate(wire:&Value,target:&ArtifactGranule,footer:&ArtifactGranule,label:&str) {
    journal::require_aggregate(wire,target,footer,label);
}

pub(super) fn require(artifact: &Value, target: &ArtifactGranule, operator: Operator, label: &str, role: &str) {
    if target.grammar != super::artifact_inventory::FrameGrammar::Common {
        return journal::require(artifact, target, operator, label, role);
    }
    let runtime = role == "runtime-integrity-observer";
    let outcome = &artifact["outcome"];
    match operator {
        Operator::CoveredByte | Operator::Checksum => {
            assert_eq!(outcome["posture"], "damaged", "{label}: {artifact}");
            assert_eq!(outcome["cause"], "checksum_mismatch", "{label}: {artifact}");
            assert_eq!(
                outcome["damaged_range"]["offset"],
                target.offset() as u64,
                "{label}"
            );
            assert_eq!(
                outcome["damaged_range"]["length"],
                target.length() as u64,
                "{label}"
            );
            assert!(
                outcome["field"].is_null(),
                "CRC mismatch cannot distinguish payload from checksum provenance"
            );
        }
        Operator::EnvelopeVersion | Operator::RecordVersion => {
            assert_eq!(outcome["posture"], "unsupported", "{label}: {artifact}");
            assert_eq!(
                outcome["observed"],
                if operator == Operator::EnvelopeVersion {
                    3
                } else {
                    2
                },
                "{label}"
            );
            assert_eq!(
                outcome["axis"],
                if operator == Operator::EnvelopeVersion {
                    "envelope_schema"
                } else {
                    if runtime { "physical_format" } else { "physical_record_format" }
                },
                "{label}"
            );
        }
        Operator::Length => {
            assert_eq!(outcome["posture"], "damaged", "{label}: {artifact}");
            assert_eq!(outcome["field"], if runtime { "encoded_length" } else { "payload_length" }, "{label}: {artifact}");
            assert_eq!(
                outcome["damaged_range"]["offset"],
                target.offset() as u64 + if runtime { 20 } else { 24 },
                "{label}"
            );
            assert_eq!(outcome["damaged_range"]["length"], if runtime { 8 } else { 4 }, "{label}");
        }
        Operator::ScopeSubstitution => {
            assert_eq!(outcome["posture"], "damaged", "{label}: {artifact}");
            assert_eq!(outcome["cause"], "scope_mismatch", "{label}: {artifact}");
        }
        Operator::Truncate => {
            assert_eq!(outcome["posture"], if runtime { "indeterminate" } else { "damaged" }, "{label}: {artifact}");
            assert_eq!(
                artifact["range"]["offset"],
                target.offset() as u64,
                "{label}"
            );
            assert_eq!(
                artifact["range"]["length"],
                target.length() as u64,
                "{label}"
            );
        }
        Operator::Pointer | Operator::Remove | Operator::Duplicate | Operator::SelectiveAggregate => unreachable!("not a common-frame row yet"),
    }
}
