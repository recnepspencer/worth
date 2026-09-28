use super::dependency_audit::{collect_file_paths, collect_file_use_paths, path_starts_with};
use super::workspace_source_inventory::WorkspaceSourceInventory;

#[path = "graph_residue_rust_source.rs"]
mod rust_source;

use rust_source::{
    collect_method_names, collect_method_names_for_function, collect_paths_for_function,
    ends_with_path, file_accepts_match_variant,
};

const DECLARATION_SEMANTIC_AUTHORITY_TYPES: &[&str] = &[
    "UiDslAspectName",
    "UiDslLoweringReceipt",
    "UiDslSemanticArtifact",
    "UiDslSemanticFamily",
    "UiDslSemanticKey",
    "UiDslStructuralToken",
    "UiDslPostureToken",
    "UiDslSupportToken",
];
const PHASE5_DECLARATION_SOURCE_REOPENING_METHODS: &[&str] = &[
    "semantic_artifact",
    "published_aspects",
    "consumed_aspects",
    "structural_tokens",
    "posture_tokens",
    "support_tokens",
];
const PHASE6_DECLARATION_SOURCE_REOPENING_METHODS: &[&str] = &[
    "semantic_artifact",
    "structural_tokens",
    "posture_tokens",
    "support_tokens",
];

pub fn audit_phase5_graph_lookup_lane_does_not_reopen_declaration_source(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let files = [
        "crates/worth-ui-runtime/src/facade/entry/app.rs",
        "crates/worth-ui-runtime/src/facade/inspection_bridge/obligation_routes.rs",
        "crates/worth-ui-runtime/src/graph/inspection/graph_lookup_boundary.rs",
        "crates/worth-ui-runtime/src/graph/inspection/graph_node_evidence_index.rs",
    ]
    .iter()
    .map(|relative| inventory.absolute_path(relative))
    .collect::<Vec<_>>();
    let mut violations = Vec::new();

    for path in files {
        for segments in collect_file_paths(inventory, &path)
            .into_iter()
            .chain(collect_file_use_paths(inventory, &path))
        {
            if let Some(authority_name) = declaration_semantic_authority_path(&segments) {
                violations.push(format!(
                    "{} reopens declaration meaning by reaching DSL semantic authority type `{authority_name}` inside the phase-5 graph lookup lane",
                    path.display()
                ));
            }
        }

        for method_name in collect_method_names(inventory, &path) {
            if PHASE5_DECLARATION_SOURCE_REOPENING_METHODS.contains(&method_name.as_str()) {
                violations.push(format!(
                    "{} reopens declaration meaning through DSL semantic accessor `{method_name}()` inside the phase-5 graph lookup lane",
                    path.display()
                ));
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_phase5_graph_lookup_lane_is_indexed_not_scan_first(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let app = inventory.absolute_path("crates/worth-ui-runtime/src/facade/entry/app.rs");
    let app_inspection_support = inventory
        .absolute_path("crates/worth-ui-runtime/src/facade/inspection_bridge/support_routing.rs");
    let graph_lookup_boundary = inventory
        .absolute_path("crates/worth-ui-runtime/src/graph/inspection/graph_lookup_boundary.rs");
    let graph_node_evidence_index = inventory
        .absolute_path("crates/worth-ui-runtime/src/graph/inspection/graph_node_evidence_index.rs");
    let obligation_inspection = inventory
        .absolute_path("crates/worth-ui-runtime/src/facade/inspection_bridge/obligation_routes.rs");
    let app_inspection_support_source = inventory.text(&app_inspection_support);
    let boundary_source = inventory.text(&graph_lookup_boundary);
    let obligation_source = inventory.text(&obligation_inspection);
    let mut violations = Vec::new();

    if !boundary_source.contains("lookup_graph_node_identity") {
        violations.push(format!(
            "{} no longer routes graph-node inspection through the graph identity evidence index",
            graph_lookup_boundary.display()
        ));
    }

    let inspect_method_names = collect_method_names_for_function(inventory, &app, "inspect");
    if inspect_method_names
        .iter()
        .any(|name| name == "rebuild_graph_node_evidence_index_from_authority")
    {
        violations.push(format!(
            "{} appears to rebuild graph-node evidence during the ordinary inspection lane instead of consuming retained derived state",
            app.display()
        ));
    }
    if collect_paths_for_function(inventory, &app, "inspect")
        .iter()
        .any(|segments| ends_with_path(segments, &["UiGraphNodeEvidenceIndex", "rebuild"]))
    {
        violations.push(format!(
            "{} appears to call `UiGraphNodeEvidenceIndex::rebuild(...)` directly inside the ordinary inspection lane instead of consuming retained derived state",
            app.display()
        ));
    }
    for (path, source) in [
        (&app_inspection_support, &app_inspection_support_source),
        (&graph_lookup_boundary, &boundary_source),
    ] {
        if source.contains("UiGraphNodeEvidenceIndex::rebuild(") {
            violations.push(format!(
                "{} appears to rebuild graph-node evidence during the ordinary inspection lane instead of consuming retained derived state",
                path.display()
            ));
        }
    }

    let method_names = collect_method_names_for_function(
        inventory,
        &graph_node_evidence_index,
        "lookup_graph_node_identity",
    );
    if !method_names.iter().any(|name| name == "get") {
        violations.push(format!(
            "{} no longer proves indexed graph lookup because `lookup_graph_node_identity` does not call map lookup `get()`",
            graph_node_evidence_index.display()
        ));
    }
    for forbidden in ["iter", "find", "position", "collect"] {
        if method_names.iter().any(|name| name == forbidden) {
            violations.push(format!(
                "{} appears to scan during ordinary graph lookup because `lookup_graph_node_identity` calls `{forbidden}()`",
                graph_node_evidence_index.display()
            ));
        }
    }

    if obligation_source.contains("declaration_artifacts().iter().find") {
        violations.push(format!(
            "{} reintroduced declaration-artifact scanning into graph-keyed obligation touch recovery",
            obligation_inspection.display()
        ));
    }
    if file_accepts_match_variant(inventory, &obligation_inspection, "GraphNodeIdentity") {
        violations.push(format!(
            "{} still accepts GraphNodeIdentity in the retained obligation helper instead of keeping ordinary graph-node lookup on the graph evidence index lane",
            obligation_inspection.display()
        ));
    }

    let graph_index_source = inventory.text(&graph_node_evidence_index);
    if !graph_index_source.contains("evidence_index()")
        || !graph_index_source.contains("record.graph_node_digest()")
    {
        violations.push(format!(
            "{} no longer proves graph-node obligation refs are rebuilt into the graph-local neighborhood from authority-backed obligation evidence records",
            graph_node_evidence_index.display()
        ));
    }
    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_phase6_aspect_lookup_lane_does_not_reopen_declaration_source(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let files = [
        "crates/worth-ui-runtime/src/facade/entry/app.rs",
        "crates/worth-ui-runtime/src/facade/inspection_bridge/support_routing.rs",
        "crates/worth-ui-runtime/src/graph/inspection/aspect/aspect_lookup_boundary.rs",
        "crates/worth-ui-runtime/src/graph/inspection/aspect/published_aspect_evidence_index.rs",
        "crates/worth-ui-runtime/src/graph/inspection/aspect/consumed_aspect_evidence_index.rs",
    ]
    .iter()
    .map(|relative| inventory.absolute_path(relative))
    .collect::<Vec<_>>();
    let mut violations = Vec::new();

    for path in files {
        for segments in collect_file_paths(inventory, &path)
            .into_iter()
            .chain(collect_file_use_paths(inventory, &path))
        {
            if let Some(authority_name) = declaration_semantic_authority_path(&segments) {
                violations.push(format!(
                    "{} reopens declaration meaning by reaching DSL semantic authority type `{authority_name}` inside the phase-6 aspect lookup lane",
                    path.display()
                ));
            }
        }

        for method_name in collect_method_names(inventory, &path) {
            if PHASE6_DECLARATION_SOURCE_REOPENING_METHODS.contains(&method_name.as_str()) {
                violations.push(format!(
                    "{} reopens declaration meaning through DSL semantic accessor `{method_name}()` inside the phase-6 aspect lookup lane",
                    path.display()
                ));
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_phase6_aspect_lookup_lane_is_indexed_not_scan_first(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let app = inventory.absolute_path("crates/worth-ui-runtime/src/facade/entry/app.rs");
    let aspect_lookup_boundary = inventory.absolute_path(
        "crates/worth-ui-runtime/src/graph/inspection/aspect/aspect_lookup_boundary.rs",
    );
    let published_aspect_evidence_index = inventory.absolute_path(
        "crates/worth-ui-runtime/src/graph/inspection/aspect/published_aspect_evidence_index.rs",
    );
    let consumed_aspect_evidence_index = inventory.absolute_path(
        "crates/worth-ui-runtime/src/graph/inspection/aspect/consumed_aspect_evidence_index.rs",
    );
    let boundary_source = inventory.text(&aspect_lookup_boundary);
    let mut violations = Vec::new();

    for required_lookup in ["lookup_published_aspect", "lookup_consumed_aspect"] {
        if !boundary_source.contains(required_lookup) {
            violations.push(format!(
                "{} no longer routes aspect inspection through `{required_lookup}` on the retained aspect evidence indexes",
                aspect_lookup_boundary.display()
            ));
        }
    }

    if collect_method_names_for_function(inventory, &app, "inspect")
        .iter()
        .any(|name| name == "build_graph_aspect_evidence_indexes")
    {
        violations.push(format!(
            "{} appears to rebuild aspect evidence during the ordinary inspection lane instead of consuming retained derived state",
            app.display()
        ));
    }
    if collect_paths_for_function(inventory, &app, "inspect")
        .iter()
        .any(|segments| ends_with_path(segments, &["UiGraphAspectEvidenceIndexes", "rebuild"]))
    {
        violations.push(format!(
            "{} appears to call `UiGraphAspectEvidenceIndexes::rebuild(...)` directly inside the ordinary aspect inspection lane instead of consuming retained derived state",
            app.display()
        ));
    }

    for path in [
        &published_aspect_evidence_index,
        &consumed_aspect_evidence_index,
    ] {
        let method_names = collect_method_names_for_function(inventory, path, "lookup");
        if !method_names.iter().any(|name| name == "get") {
            violations.push(format!(
                "{} no longer proves indexed aspect lookup because `lookup` does not call map lookup `get()`",
                path.display()
            ));
        }
        for forbidden in ["iter", "find", "position", "collect"] {
            if method_names.iter().any(|name| name == forbidden) {
                violations.push(format!(
                    "{} appears to scan during ordinary aspect lookup because `lookup` calls `{forbidden}()`",
                    path.display()
                ));
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

fn declaration_semantic_authority_path(segments: &[String]) -> Option<&str> {
    if !path_starts_with(segments, "worth_ui_dsl") {
        return None;
    }

    DECLARATION_SEMANTIC_AUTHORITY_TYPES
        .iter()
        .copied()
        .find(|name| segments.iter().any(|segment| segment == name))
}
