use std::path::Path;

use crate::{repository_document, workspace_source_inventory};

const RUNTIME_ROOT: &str = "crates/worth-ui-runtime/src";
const SCOPE_RESOLVER: &str = "crates/worth-ui-runtime/src/runtime/rebind/scope/resolver.rs";
const SUBSYSTEM_COMPILER: &str =
    "crates/worth-ui-runtime/src/runtime/rebind/planning/subsystem_compiler.rs";
const PLAN_COMPILER: &str = "crates/worth-ui-runtime/src/runtime/rebind/planning/compiler.rs";
const PLAN_TYPE: &str = "crates/worth-ui-runtime/src/runtime/rebind/planning/plan.rs";
const SESSION_ENTRY: &str =
    "crates/worth-ui-runtime/src/runtime/session/application_state/rebind_planning.rs";
const REPLACEMENT_ORCHESTRATOR: &str =
    "crates/worth-ui-runtime/src/runtime/replacement/orchestrator.rs";

#[test]
fn milestone_312_phase3_r12_has_one_canonical_rebind_planning_authority() {
    let inventory = workspace_source_inventory();
    let phase_1: toml::Value = toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.12-phase-1-contract.toml",
    ))
    .expect("Phase 1 contract is TOML");
    let route = phase_1["route"]
        .as_array()
        .expect("route inventory")
        .iter()
        .find(|route| route["id"].as_str() == Some("R-12"))
        .expect("R-12 is inventoried");
    assert_eq!(route["disposition"].as_str(), Some("canonical cutover"));
    assert_eq!(route["cutover_phase"].as_integer(), Some(3));
    assert_eq!(
        route["successor_home"].as_str(),
        Some("fact_contract plus runtime/rebind/scope and planning")
    );

    let scope = inventory.text(SCOPE_RESOLVER);
    for required in [
        "UiGraphFactIndexBasis",
        "lookup_both_generations",
        "predecessor_lookup",
        "candidate_lookup",
        "UiResolvedAffectedScope::new",
    ] {
        assert!(
            scope.contains(required),
            "R-12 successor scope must retain `{required}`"
        );
    }
    for obsolete in ["narrow_resolved_frame", "UiNarrowedAllocationFramePlan"] {
        assert!(
            !scope.contains(obsolete),
            "R-12 successor must not wrap predecessor terminal `{obsolete}`"
        );
    }

    let subsystem = inventory.text(SUBSYSTEM_COMPILER);
    for required in [
        "UiRebindSubsystemKind::Allocation",
        "allocation_family",
        "UiRebindSubsystemPlan::new",
    ] {
        assert!(
            subsystem.contains(required),
            "canonical planner must own allocation decision `{required}`"
        );
    }

    let compiler = inventory.text(PLAN_COMPILER);
    for required in [
        "compile_subsystems",
        "finish_precomputed_replacement_lowering",
        "UiRebindPlan::new",
    ] {
        assert!(
            compiler.contains(required),
            "canonical compiler must consume proof through `{required}`"
        );
    }
    assert!(
        inventory
            .text(REPLACEMENT_ORCHESTRATOR)
            .contains("WorthUiReplacementLoweringReady"),
        "replacement remains a typed proof producer"
    );
    for forbidden in [
        "commit_application_activation",
        "publish_rebind",
        "execute_rebind",
    ] {
        assert!(
            !inventory.text(REPLACEMENT_ORCHESTRATOR).contains(forbidden),
            "replacement orchestrator retained competing authority `{forbidden}`"
        );
    }

    assert_eq!(
        production_locations("UiRebindPlan::new("),
        vec![PLAN_COMPILER.to_owned()],
        "only the canonical compiler may construct a rebind plan"
    );
    assert!(
        inventory.text(PLAN_TYPE).contains("pub(crate) fn new("),
        "plan construction must remain crate-private"
    );
    assert!(
        inventory
            .text(SESSION_ENTRY)
            .contains("UiRebindPlanCompiler::compile("),
        "the session entry must delegate to the canonical compiler"
    );
}

fn production_locations(needle: &str) -> Vec<String> {
    workspace_source_inventory()
        .rust_files_under(RUNTIME_ROOT)
        .filter(|source| !is_test_source(source.relative_path()))
        .filter(|source| source.text().contains(needle))
        .map(|source| source.relative_path().to_string_lossy().replace('\\', "/"))
        .collect()
}

fn is_test_source(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
        || path
            .file_stem()
            .is_some_and(|stem| stem.to_string_lossy().ends_with("_tests"))
}
