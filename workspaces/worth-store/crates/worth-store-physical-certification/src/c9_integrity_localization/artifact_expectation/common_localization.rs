//! Independently declared field coordinates and minimum honest scope of damage.
use super::{ArtifactGranule, Value};
use serde_json::json;

pub(super) fn scope(row: &Value, target: &ArtifactGranule, runtime: bool) {
    let (field, offset, length, blast) = match (target.family, runtime) {
        ("bootstrap_catalog" | "current_root_selector" | "previous_root_selector", true) => {
            ("store_identity", 48, 16, "artifact")
        }
        ("bootstrap_catalog" | "current_root_selector" | "previous_root_selector", false) => {
            ("store_identity", 48, 16, "field")
        }
        ("root_manifest", true) => ("physical_generation", 28, 8, "reachable_root_subtree"),
        ("root_manifest", false) => ("manifest_generation", 48, 8, "field"),
        (
            "root_routing_block" | "segment_membership_block" | "free_space_membership_block",
            true,
        ) => ("physical_generation", 72, 8, "reachable_root_subtree"),
        (
            "root_routing_block" | "segment_membership_block" | "free_space_membership_block",
            false,
        ) => ("manifest_generation", 72, 8, "field"),
        ("free_space_header", true) => ("physical_generation", 28, 8, "reachable_root_subtree"),
        ("free_space_header", false) => ("manifest_generation", 28, 28, "field"),
        ("extent_manifest", true) => ("physical_generation", 28, 8, "reachable_root_subtree"),
        ("extent_manifest", false) => ("frame_identity", 28, 8, "field"),
        ("extent_chunk", true) => ("physical_generation", 80, 8, "artifact"),
        ("extent_chunk", false) => ("manifest_generation", 80, 8, "field"),
        // S selects a different page in the same real segment, not arbitrary bytes.
        ("inline_page", true) => ("page_identity", 56, 8, "artifact"),
        ("inline_page", false) => ("identity_field", 48, 16, "field"),
        _ => unreachable!("closed common-family matrix"),
    };
    damage(row, target, field, offset, length, blast);
}

pub(super) fn pointer(row: &Value, target: &ArtifactGranule, runtime: bool) {
    let (field, offset, length, blast) = match (target.family, runtime) {
        ("current_root_selector" | "previous_root_selector", true) => {
            ("root_generation", 65, 8, "reachable_root_subtree")
        }
        ("current_root_selector" | "previous_root_selector", false) => {
            ("root_generation", 65, 8, "field")
        }
        ("root_manifest", true) => ("child_reference", 96, 8, "reachable_root_subtree"),
        ("root_manifest", false) => ("manifest_pointer", 96, 72, "reachable_root_subtree"),
        ("root_routing_block" | "segment_membership_block", true) => {
            ("child_reference", 88, 8, "reachable_root_subtree")
        }
        ("root_routing_block" | "segment_membership_block", false) => {
            ("manifest_pointer", 88, 20, "field")
        }
        ("free_space_header", true) => ("child_reference", 112, 64, "reachable_root_subtree"),
        ("free_space_header", false) => ("manifest_pointer", 120, 56, "field"),
        ("free_space_membership_block", true) => (
            "membership_range",
            88,
            target.length() - 88,
            "reachable_root_subtree",
        ),
        ("free_space_membership_block", false) => {
            ("manifest_pointer", 88, target.length() - 88, "field")
        }
        ("extent_manifest", true) => ("extent_identity", 72, 8, "reachable_root_subtree"),
        ("extent_manifest", false) => ("identity_field", 48, 32, "field"),
        _ => unreachable!("P has no row for pointer-free families"),
    };
    damage(row, target, field, offset, length, blast);
}

fn damage(
    row: &Value,
    target: &ArtifactGranule,
    field: &str,
    offset: usize,
    length: usize,
    blast: &str,
) {
    let outcome = &row["outcome"];
    assert_eq!(outcome["field"], field, "{row}");
    assert_eq!(
        outcome["damaged_range"],
        json!({"offset":target.offset()+offset,"length":length}),
        "{row}"
    );
    assert_eq!(outcome["blast_radius"], blast, "{row}");
}
