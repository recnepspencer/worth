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
