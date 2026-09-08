//! Expectations name the invariant made false by each declared clean-world edit.
//! Runtime and independent readers have different diagnostic vocabularies.
use super::{ArtifactGranule, Value};

pub(super) fn require_scope(row: &Value, target: &ArtifactGranule, runtime: bool, label: &str) {
    let outcome = &row["outcome"];
    assert_eq!(outcome["posture"], "damaged", "{label}: {row}");
    let cause = if !runtime {
        "scope_mismatch"
    } else {
        match target.family {
            "bootstrap_catalog" | "current_root_selector" | "previous_root_selector" => {
                "store_identity_mismatch"
            }
            "root_manifest"
            | "root_routing_block"
            | "segment_membership_block"
            | "free_space_header"
            | "free_space_membership_block"
            | "extent_manifest" => "physical_generation_mismatch",
            "inline_page" => "artifact_identity_mismatch",
            "extent_chunk" => "physical_generation_mismatch",
            family => panic!("missing scope expectation for {family}"),
        }
    };
    assert_eq!(outcome["cause"], cause, "{label}: {row}");
    super::common_localization::scope(row, target, runtime);
    // Every observation must retain the parent's original concrete target range,
    // never replace it with the substituted artifact's self-declared coordinates.
    assert_eq!(row["range"]["offset"], target.offset() as u64, "{label}");
    assert_eq!(row["range"]["length"], target.length() as u64, "{label}");
}

pub(super) fn require_pointer(row: &Value, target: &ArtifactGranule, runtime: bool, label: &str) {
    let outcome = &row["outcome"];
    assert_eq!(outcome["posture"], "damaged", "{label}: {row}");
    let (cause, field) = match (target.family, runtime) {
        ("current_root_selector" | "previous_root_selector", true) => {
            ("physical_generation_mismatch", "root_generation")
        }
        ("current_root_selector" | "previous_root_selector", false) => {
            ("pointer", "root_generation")
        }
        ("root_manifest", true) => ("child_reference_mismatch", "child_reference"),
        ("root_manifest", false) => ("pointer", "manifest_pointer"),
        ("root_routing_block" | "segment_membership_block" | "free_space_header", true) => {
            ("child_reference_mismatch", "child_reference")
        }
        (
            "root_routing_block"
            | "segment_membership_block"
            | "free_space_header"
            | "free_space_membership_block",
            false,
        ) => ("scope_mismatch", "manifest_pointer"),
        ("free_space_membership_block", true) => ("child_reference_mismatch", "membership_range"),
        ("extent_manifest", true) => ("artifact_identity_mismatch", "extent_identity"),
        ("extent_manifest", false) => ("scope_mismatch", "identity_field"),
        (family, _) => panic!("missing pointer expectation for {family}"),
    };
    assert_eq!(outcome["cause"], cause, "{label}: {row}");
    assert_eq!(outcome["field"], field, "{label}: {row}");
    super::common_localization::pointer(row, target, runtime);
    assert_eq!(row["range"]["offset"], target.offset() as u64, "{label}");
    assert_eq!(row["range"]["length"], target.length() as u64, "{label}");
}
