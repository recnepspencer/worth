//! Fixed expectations derive from the named edit and canonical field coordinates.
//! Runtime/offline vocabulary differences are evidence, not forcibly normalized.
use super::{ArtifactGranule, Operator as Op};
use serde_json::{json, Value};

pub(super) fn require(
    artifact: &Value,
    target: &ArtifactGranule,
    operator: Op,
    label: &str,
    role: &str,
) {
    let runtime = role == "runtime-integrity-observer";
    let wal = target.family == "wal_frame";
    let outcome = &artifact["outcome"];
    let start = target.offset() as u64;
    let length = target.length() as u64;
    match operator {
        Op::CoveredByte | Op::Checksum => {
            let (offset, count, field, blast) = if wal {
                if runtime && operator == Op::Checksum {
                    (start + 84, 32, Some("checksum"), "field")
                } else if runtime {
                    (start, length, None, "frame")
                } else {
                    (start + 116, length - 148, None, "frame")
                }
            } else {
                (
                    start,
                    if runtime { length } else { length - 4 },
                    None,
                    "frame",
                )
            };
            damage(
                outcome,
                "checksum_mismatch",
                offset,
                count,
                field,
                blast,
                label,
            );
        }
        Op::Length => {
            let (offset, count) = if wal { (10, 2) } else { (12, 4) };
            damage(
                outcome,
                if runtime {
                    "framing_length_mismatch"
                } else {
                    "framing"
                },
                start + offset,
                count,
                Some(if runtime {
                    "encoded_length"
                } else {
                    "payload_length"
                }),
                if runtime { "frame" } else { "field" },
                label,
            );
        }
        Op::EnvelopeVersion => {
            assert_eq!(outcome["posture"], "unsupported", "{label}: {artifact}");
            assert_eq!(outcome["observed"], 2, "{label}");
            assert_eq!(outcome["supported"], "1", "{label}");
            assert_eq!(
                outcome["axis"],
                if wal {
                    "wal_frame"
                } else if runtime {
                    "checkpoint_record_schema"
                } else {
                    "checkpoint_record"
                },
                "{label}"
            );
            assert_eq!(
                outcome["range"],
                if runtime {
                    json!({"offset":start,"length":length})
                } else {
                    json!({"offset":start+8,"length":if wal{2}else{1}})
                },
                "{label}"
            );
        }
        Op::ScopeSubstitution if wal => {
            damage(
                outcome,
                if runtime {
                    "artifact_identity_mismatch"
                } else {
                    "scope_mismatch"
                },
                start + 12,
                if runtime { 8 } else { 16 },
                Some(if runtime {
                    "segment_identity"
                } else {
                    "identity_field"
                }),
                if runtime { "artifact" } else { "field" },
                label,
            );
        }
        Op::ScopeSubstitution if target.family == "checkpoint_footer" => {
            damage(
                outcome,
                if runtime {
                    "sequence_mismatch"
                } else {
                    "scope_mismatch"
                },
                start + if runtime { 32 } else { 16 },
                if runtime { 8 } else { 24 },
                Some(if runtime {
                    "checkpoint_identity"
                } else {
                    "identity_field"
                }),
                if runtime { "artifact" } else { "field" },
                label,
            );
        }
        Op::ScopeSubstitution => {
            damage(
                outcome,
                if runtime {
                    "record_kind_mismatch"
                } else {
                    "scope_mismatch"
                },
                start + 9,
                1,
                Some(if runtime {
                    "checkpoint_record_kind"
                } else {
                    "family_kind"
                }),
                if runtime { "artifact" } else { "field" },
                label,
            );
            if runtime && target.family == "checkpoint_binding" {
                assert_eq!(
                    artifact["range"],
                    json!({"offset":start,"length":36}),
                    "{label}"
                );
            }
        }
        Op::SelectiveAggregate => {
            // The selected body is locally intact. The separate footer assertion
            // below must convict the still-false selective summary.
            if target.family == "checkpoint_footer" {
                aggregate(outcome, start + 48, runtime, label);
            } else {
                assert_eq!(outcome["posture"], "intact", "{label}: {artifact}");
            }
        }
        Op::Truncate if runtime => {
            assert_eq!(outcome["posture"], "indeterminate", "{label}: {artifact}");
            assert_eq!(
                artifact["range"],
                json!({"offset":start,"length":length}),
                "{label}"
            );
            assert_eq!(
                outcome["observed_range"],
                json!({"offset":start,"length":length/2}),
                "{label}"
            );
        }
        Op::Truncate => {
            damage(
                outcome,
                "truncation",
                start + length / 2,
                length - length / 2,
                None,
                "frame",
                label,
            );
        }
        _ => unreachable!("journal applicability is closed"),
    }
}

pub(in crate::c9_integrity_localization) fn require_aggregate(
    wire: &Value,
    target: &ArtifactGranule,
    footer: &ArtifactGranule,
    label: &str,
) {
    let path = footer.path.to_string_lossy().replace('\\', "/");
    let artifact = wire["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == path && row["range"]["offset"] == footer.offset() as u64)
        .unwrap();
    let offset = footer.offset() as u64
        + if target.family == "checkpoint_binding" {
            120
        } else {
            48
        };
    aggregate(
        &artifact["outcome"],
        offset,
        wire["role"] == "runtime-integrity-observer",
        label,
    );
}

fn aggregate(outcome: &Value, offset: u64, runtime: bool, label: &str) {
    damage(
        outcome,
        if runtime {
            "aggregate_mismatch"
        } else {
            "checksum_mismatch"
        },
        offset,
        32,
        if runtime {
            Some("checkpoint_aggregate")
        } else {
            None
        },
        "artifact",
        label,
    );
}
fn damage(
    outcome: &Value,
    cause: &str,
    offset: u64,
    length: u64,
    field: Option<&str>,
    blast: &str,
    label: &str,
) {
    assert_eq!(outcome["posture"], "damaged", "{label}: {outcome}");
    assert_eq!(outcome["cause"], cause, "{label}: {outcome}");
    assert_eq!(
        outcome["damaged_range"],
        json!({"offset":offset,"length":length}),
        "{label}: {outcome}"
    );
    assert_eq!(outcome["field"], json!(field), "{label}: {outcome}");
    assert_eq!(outcome["blast_radius"], blast, "{label}: {outcome}");
}
