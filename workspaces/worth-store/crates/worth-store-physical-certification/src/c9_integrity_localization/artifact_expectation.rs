use super::{artifact_edit::ArtifactOperator as Operator, artifact_inventory::ArtifactGranule};
use serde_json::Value;

mod common_localization;
mod common_scope;
mod common_semantics;
pub(super) use common_scope::require as require_common_scope;
mod journal;

pub(super) fn require_aggregate(
    wire: &Value,
    target: &ArtifactGranule,
    footer: &ArtifactGranule,
    label: &str,
) {
    journal::require_aggregate(wire, target, footer, label);
}

pub(super) fn require(
    artifact: &Value,
    target: &ArtifactGranule,
    operator: Operator,
    label: &str,
    role: &str,
) {
    if target.grammar != super::artifact_inventory::FrameGrammar::Common {
        return journal::require(artifact, target, operator, label, role);
    }
    let runtime = role == "runtime-integrity-observer";
    let outcome = &artifact["outcome"];
    match operator {
        Operator::CoveredByte | Operator::Checksum => {
            assert_eq!(outcome["blast_radius"], "frame", "{artifact}");
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
            assert_eq!(
                outcome["supported"],
                if operator == Operator::EnvelopeVersion {
                    "2"
                } else {
                    "1"
                }
            );
            let (offset, length) = if runtime {
                (target.offset(), target.length())
            } else if operator == Operator::EnvelopeVersion {
                (target.offset() + 9, 1)
            } else {
                (target.offset() + 10, 2)
            };
            assert_eq!(
                outcome["range"],
                serde_json::json!({"offset":offset,"length":length}),
                "{artifact}"
            );
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
                    if runtime {
                        "physical_format"
                    } else {
                        "physical_record_format"
                    }
                },
                "{label}"
            );
        }
        Operator::Length => {
            assert_eq!(
                outcome["cause"],
                if runtime {
                    "framing_length_mismatch"
                } else {
                    "framing"
                },
                "{artifact}"
            );
            assert_eq!(
                outcome["blast_radius"],
                if runtime { "frame" } else { "field" },
                "{artifact}"
            );
            assert_eq!(outcome["posture"], "damaged", "{label}: {artifact}");
            assert_eq!(
                outcome["field"],
                if runtime {
                    "encoded_length"
                } else {
                    "payload_length"
                },
                "{label}: {artifact}"
            );
            assert_eq!(
                outcome["damaged_range"]["offset"],
                target.offset() as u64 + if runtime { 20 } else { 24 },
                "{label}"
            );
            assert_eq!(
                outcome["damaged_range"]["length"],
                if runtime { 8 } else { 4 },
                "{label}"
            );
        }
        Operator::ScopeSubstitution => {
            common_semantics::require_scope(artifact, target, runtime, label);
        }
        Operator::Truncate => {
            assert_eq!(
                outcome["posture"],
                if runtime { "indeterminate" } else { "damaged" },
                "{label}: {artifact}"
            );
            // Concrete address and the independent reader's surviving tree
            // prefix are checked together by require_common_scope.
            let prefix = target.length() / 2;
            if runtime {
                assert_eq!(outcome["reason"], "stable_range_not_proven", "{artifact}");
                assert_eq!(
                    outcome["observed_range"],
                    serde_json::json!({"offset":target.offset(),"length":prefix}),
                    "{artifact}"
                );
            } else {
                // The independent page walk first checks the parent-declared
                // segment length. A short container fails that whole-file
                // relation before it attempts to acquire the final page.
                let page_container = target.family == "inline_page";
                assert_eq!(
                    outcome["cause"],
                    if page_container {
                        "framing"
                    } else {
                        "truncation"
                    },
                    "{artifact}"
                );
                assert!(outcome["field"].is_null());
                assert_eq!(outcome["blast_radius"], "artifact", "{artifact}");
                assert_eq!(
                    outcome["damaged_range"],
                    if page_container {
                        serde_json::json!({"offset":0,"length":target.offset()+prefix})
                    } else {
                        serde_json::json!({"offset":target.offset()+prefix,"length":target.length()-prefix})
                    },
                    "{artifact}"
                );
            }
        }
        Operator::Pointer => common_semantics::require_pointer(artifact, target, runtime, label),
        Operator::Remove | Operator::Duplicate | Operator::SelectiveAggregate => {
            unreachable!("not a common-frame row")
        }
    }
}
