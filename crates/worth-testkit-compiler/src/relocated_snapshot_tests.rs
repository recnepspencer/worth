//! Renderer regressions use complete structural fixture and source identities.
use std::path::{Path, PathBuf};

use crate::{snapshot::compared, source_locations::normalize};

const COURT: &str = "worth_store_court_store_physical_record_journeys_physical_work";

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("workspaces/worth-store")
}

fn source() -> PathBuf {
    workspace()
        .join("crates/worth-store-court-store-physical-record-journeys-physical-work/tests")
        .join(COURT)
        .join("sources/worth_store/physical_runtime_authority/fixture.rs")
}

#[test]
fn moved_relative_fixture_paths_require_the_exact_current_court_and_owner() {
    let moved = format!(
        " --> tests/{COURT}/sources/worth_store/physical_runtime_authority/fixture.rs:11:33"
    );
    let absolute = format!(" --> {}:11:33", source().display());
    let canonical = normalize(&absolute, &source());
    assert_eq!(normalize(&moved, &source()), canonical);
    for changed in [
        moved.replace(COURT, "another_court"),
        moved.replace("sources/worth_store/", "sources/worth_other/"),
        moved.replace("physical_runtime_authority/", "another_directory/"),
        moved.replace("fixture.rs:11:33", "fixture.rs:12:33"),
    ] {
        assert_ne!(normalize(&changed, &source()), canonical);
    }
}

#[test]
fn symbolic_workspace_paths_keep_the_exact_source_owner_and_module_identity() {
    let symbolic = " --> $WORKSPACE/crates/worth-store/src/physical_runtime/blob/facade.rs";
    let absolute = format!(
        " --> {}",
        workspace()
            .join("crates/worth-store/src/physical_runtime/blob/facade.rs")
            .display()
    );
    let canonical = normalize(&absolute, &source());
    assert_eq!(normalize(symbolic, &source()), canonical);
    assert_eq!(canonical, " --> src/physical_runtime/blob/facade.rs");
    for changed in [
        symbolic.replace("crates/worth-store/", "crates/worth-other/"),
        symbolic.replace("blob/facade.rs", "another/facade.rs"),
    ] {
        assert_ne!(normalize(&changed, &source()), canonical);
    }
}

fn renderer_block(primary_margin: &str, gutter: &str, line: &str, external: &str) -> String {
    format!(
        "error[E0308]: mismatched types\n{primary_margin}--> {}:21:33\n{gutter}|\n{line} |     sealed();\n{gutter}|     ^^^^^^ wrong argument\n{gutter}|\nnote: function defined here\n{primary_margin}--> {external}\n{gutter}|\n{gutter}| pub fn sealed()\n{gutter}|        ^^^^^^",
        source().display()
    )
}

#[test]
fn external_gutters_follow_each_error_block_after_common_margin_removal() {
    let external = workspace().join("crates/worth-store/src/physical_runtime/blob/facade.rs");
    let expected = renderer_block("  ", "   ", "21", "src/physical_runtime/blob/facade.rs");
    let actual = renderer_block(
        "   ",
        "    ",
        " 21",
        &format!("{}:143:8", external.display()),
    )
    .replace("    | pub fn", "143 | pub fn");
    // The second block has a wider compiler gutter. Its source and labels must
    // compare identically without borrowing the first block's indentation.
    let expected = format!("{expected}\n\n{expected}");
    let actual = format!(
        "{}\n\n{actual}",
        renderer_block("  ", "   ", "21", &format!("{}:87:8", external.display()))
            .replace("   | pub fn", "87 | pub fn")
    );
    let (normalized, observed) = compared(&expected, &actual, &source()).unwrap();
    assert_eq!(normalized, observed);
    for changed in [
        actual.replace("|     sealed", "|    sealed"),
        actual.replace("^^^^^^ wrong", "^^^^^ wrong"),
        actual.replace("wrong argument", "different denial"),
        actual.replace("facade.rs", "another.rs"),
        actual.replace(":21:33", ":21:34"),
    ] {
        let (expected, observed) = compared(&expected, &changed, &source()).unwrap();
        assert_ne!(expected, observed);
    }
}

#[test]
fn private_multiline_secondary_definitions_preserve_all_text_after_the_gutter() {
    let expected = "error[E0624]: private method\n --> tests/physical_runtime_authority/fixture.rs:8:54\n  |\n8 |     sealed();\n  |     ^^^^^^ private method\n  |\n ::: src/physical_runtime/private.rs\n  |\n  | / pub fn sealed(\n  | |     value: u64,\n  | | ) {}\n  | |____- private method defined here";
    let actual = format!("error[E0624]: private method\n  --> {}:8:54\n   |\n 8 |     sealed();\n   |     ^^^^^^ private method\n   |\n  ::: {}:76:5\n   |\n76 | / pub fn sealed(\n77 | |     value: u64,\n78 | | ) {{}}\n   | |____- private method defined here", source().display(), workspace().join("crates/worth-store/src/physical_runtime/private.rs").display());
    let (normalized, observed) = compared(expected, &actual, &source()).unwrap();
    assert_eq!(normalized, observed);
    for changed in [
        actual.replace("| |     value", "| |    value"),
        actual.replace("____-", "___-"),
        actual.replace("private method defined here", "different label"),
        actual.replace("private.rs", "another.rs"),
    ] {
        let (expected, observed) = compared(expected, &changed, &source()).unwrap();
        assert_ne!(expected, observed);
    }
}

fn recovery_source() -> PathBuf {
    workspace().join("crates/worth-store-court-recovery-runtime/tests/worth_store_court_recovery_runtime/sources/worth_store_recovery_runtime/ui/fixture.rs")
}

const BUDGET_MODULE: &str =
    "src/orchestration/planning/page_observation/walk_entry/manifest_entry_budget.rs";

fn moved_budget_module() -> PathBuf {
    recovery_source().parent().unwrap().join(format!(
        "manifest_entry/../../../../../../../worth-store-recovery-runtime/{BUDGET_MODULE}"
    ))
}

#[test]
fn parent_module_locations_resolve_the_complete_current_and_original_owner_roots() {
    let original = format!(" ::: tests/ui/manifest_entry/../../../{BUDGET_MODULE}");
    let moved = format!(" ::: {}", moved_budget_module().display());
    let current_relative = format!(" ::: tests/worth_store_court_recovery_runtime/sources/worth_store_recovery_runtime/ui/manifest_entry/../../../../../../../worth-store-recovery-runtime/{BUDGET_MODULE}");
    let canonical = format!(" ::: {BUDGET_MODULE}");
    assert_eq!(normalize(&original, &recovery_source()), canonical);
    assert_eq!(normalize(&moved, &recovery_source()), canonical);
    assert_eq!(normalize(&current_relative, &recovery_source()), canonical);
    for changed in [
        moved.replace(
            "worth-store-recovery-runtime",
            "worth-store-physical-format",
        ),
        moved.replace("manifest_entry_budget.rs", "another_budget.rs"),
        original.replace("page_observation/", "another_module/"),
    ] {
        assert_ne!(normalize(&changed, &recovery_source()), canonical);
    }
}

#[test]
fn parent_module_definitions_are_external_and_fixture_coordinates_stay_asserted() {
    let expected = format!("error[E0624]: private function\n  --> tests/ui/fixture.rs:10:34\n   |\n10 |     budget();\n   |     ^^^^^^ private function\n   |\n  ::: tests/ui/manifest_entry/../../../{BUDGET_MODULE}\n   |\n   | pub(super) fn budget() {{}}\n   | ------------------------- private function defined here");
    let actual = format!("error[E0624]: private function\n   --> {}:10:34\n    |\n 10 |     budget();\n    |     ^^^^^^ private function\n    |\n   ::: {}:134:5\n    |\n134 | pub(super) fn budget() {{}}\n    | ------------------------- private function defined here", recovery_source().display(), moved_budget_module().display());
    let (normalized, observed) = compared(&expected, &actual, &recovery_source()).unwrap();
    assert_eq!(normalized, observed);
    for changed in [
        actual.replace(":10:34", ":11:34"),
        actual.replace("|     budget", "|    budget"),
        actual.replace("private function defined here", "different label"),
        actual.replace("manifest_entry_budget.rs", "another_budget.rs"),
    ] {
        let (expected, observed) = compared(&expected, &changed, &recovery_source()).unwrap();
        assert_ne!(expected, observed);
    }
    let asserted = expected.replace(BUDGET_MODULE, &format!("{BUDGET_MODULE}:133:5"));
    let (expected, observed) = compared(&asserted, &actual, &recovery_source()).unwrap();
    assert_ne!(expected, observed);
}

#[test]
fn referenced_fixture_modules_hide_only_unasserted_secondary_coordinates() {
    let module = source().parent().unwrap().join("support/walk_entry.rs");
    let expected = "error[E0382]: use of moved value\n  --> tests/physical_runtime_authority/fixture.rs:15:35\n   |\n15 |     take(charge);\n   |          ^^^^^^ moved value\n   |\nnote: function takes ownership\n  --> tests/physical_runtime_authority/support/walk_entry.rs\n   |\n   | pub fn take(charge: Charge) {}\n   |        ----         ^^^^^^ takes ownership";
    let actual = format!("error[E0382]: use of moved value\n  --> {}:15:35\n   |\n15 |     take(charge);\n   |          ^^^^^^ moved value\n   |\nnote: function takes ownership\n  --> {}:23:26\n   |\n23 | pub fn take(charge: Charge) {{}}\n   |        ----         ^^^^^^ takes ownership", source().display(), module.display());
    let (normalized, observed) = compared(expected, &actual, &source()).unwrap();
    assert_eq!(normalized, observed);
    for changed in [
        actual.replace(":15:35", ":16:35"),
        actual.replace("walk_entry.rs", "another_module.rs"),
        actual.replace("support", "another_directory"),
        actual.replace("| pub fn take", "|  pub fn take"),
        actual.replace("^^^^^^ moved", "^^^^^ moved"),
        actual.replace("takes ownership", "different label"),
        actual.replace("E0382", "E0507"),
    ] {
        let (expected, observed) = compared(expected, &changed, &source()).unwrap();
        assert_ne!(expected, observed);
    }
    let asserted = expected.replace("walk_entry.rs", "walk_entry.rs:24:26");
    let (expected, observed) = compared(&asserted, &actual, &source()).unwrap();
    assert_ne!(expected, observed);
}

#[test]
fn primary_fixture_definitions_never_use_the_secondary_module_coordinate_mask() {
    let expected = "error: denied\n --> tests/physical_runtime_authority/fixture.rs:15:35\n  |\n15 |     take(charge);\n  |          ^^^^^^ denied\nnote: function defined here\n --> tests/physical_runtime_authority/fixture.rs\n  |\n  | fn take(charge: Charge) {}";
    let actual = format!("error: denied\n --> {}:15:35\n  |\n15 |     take(charge);\n  |          ^^^^^^ denied\nnote: function defined here\n --> {}:4:4\n  |\n4 | fn take(charge: Charge) {{}}", source().display(), source().display());
    let (expected, observed) = compared(expected, &actual, &source()).unwrap();
    assert_ne!(expected, observed);
    assert!(observed.contains("fixture.rs:4:4"));
}

#[test]
fn support_module_coordinate_mask_requires_a_structurally_identified_fixture_tree() {
    let expected = "error: denied\n --> tests/support/module.rs\n  |\n  | fn take() {}";
    let observed = "error: denied\n --> tests/support/module.rs:4:4\n  |\n4 | fn take() {}";
    let (expected, observed) = compared(expected, observed, Path::new("fixture.rs")).unwrap();
    assert_ne!(expected, observed);
}
