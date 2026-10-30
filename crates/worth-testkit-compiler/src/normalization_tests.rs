//! Synthetic rendering tests; actual denial evidence belongs to consuming courts.
use std::path::{Path, PathBuf};

use crate::{snapshot::compared, source_locations::normalize};

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/owner_court/sources/worth_foundational/ui/fixture.rs")
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn asset_fixtures_retain_their_original_complete_test_path_and_coordinates() {
    let fixture =
        workspace().join("crates/worth-testkit-foundational/assets/ui/contracts/forged.rs");
    let actual = format!(
        "error: denied\n --> {}:42:9\n42 | forged();\n   | ^^^^^^ denied",
        fixture.display()
    );
    let expected =
        "error: denied\n --> tests/ui/contracts/forged.rs:42:9\n42 | forged();\n   | ^^^^^^ denied";
    let (expected, actual) = compared(expected, &actual, &fixture).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn external_definitions_hide_only_coordinates_and_gutters_the_snapshot_does_not_assert() {
    let actual = format!("error: denied\n --> {}:919:4\n    |\n919 | pub fn sealed() {{}}\n    |    ^^^^^^ defined here", workspace().join("crates/worth-foundational/src/contracts.rs").display());
    let expected = "error: denied\n --> src/contracts.rs\n  |\n  | pub fn sealed() {}\n  |    ^^^^^^ defined here";
    let (expected, actual) = compared(expected, &actual, &source()).unwrap();
    assert_eq!(expected, actual);
    let changed = actual.replace("sealed", "open");
    assert_ne!(expected, changed);
}

#[test]
fn asserted_external_coordinates_are_never_discarded() {
    let actual = format!(
        "error: denied\n --> {}:919:4\n919 | pub fn sealed() {{}}",
        workspace()
            .join("crates/worth-foundational/src/contracts.rs")
            .display()
    );
    let expected = "error: denied\n --> src/contracts.rs:918:4\n918 | pub fn sealed() {}";
    let (expected, actual) = compared(expected, &actual, &source()).unwrap();
    assert_ne!(expected, actual);
    assert!(actual.contains(":919:4"));
}

#[test]
fn test_fixture_coordinates_cannot_use_the_external_definition_mask() {
    let actual = format!(
        "error: denied\n --> {}:9:4\n9 | sealed();",
        source().display()
    );
    let expected = "error: denied\n --> tests/ui/fixture.rs\n  | sealed();";
    let (expected, actual) = compared(expected, &actual, &source()).unwrap();
    assert_ne!(expected, actual);
    assert!(actual.contains(":9:4"));
}

#[test]
fn a_different_external_file_is_not_accepted_by_basename() {
    let actual = format!(
        "error: denied\n --> {}:9:4\n9 | sealed();",
        workspace()
            .join("crates/worth-foundational/src/other/contracts.rs")
            .display()
    );
    let expected = "error: denied\n --> src/contracts.rs\n  | sealed();";
    let (expected, actual) = compared(expected, &actual, &source()).unwrap();
    assert_ne!(expected, actual);
    assert!(actual.contains("other/contracts.rs:9:4"));
}

#[test]
fn dependency_locations_keep_their_full_workspace_path() {
    let actual = format!(
        "error: denied\n --> {}:9:4\n9 | sealed();",
        workspace()
            .join("crates/worth-proof/src/proof/marker_authoring.rs")
            .display()
    );
    let expected = "error: denied\n --> $WORKSPACE/crates/worth-proof/src/proof/marker_authoring.rs\n  | sealed();";
    let (expected, actual) = compared(expected, &actual, &source()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn registry_versions_are_masked_only_when_the_snapshot_uses_a_version_placeholder() {
    let actual = "error: denied\n --> /cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.149/src/de.rs:99:2\n99 | sealed();";
    let expected = "error: denied\n --> $CARGO/serde_json-$VERSION/src/de.rs\n   | sealed();";
    let (expected, observed) = compared(expected, actual, &source()).unwrap();
    assert_eq!(expected, observed);
    let pinned = "error: denied\n --> $CARGO/serde_json-1.0.148/src/de.rs\n   | sealed();";
    let (expected, observed) = compared(pinned, actual, &source()).unwrap();
    assert_ne!(expected, observed);
    let different = actual.replace("serde_json-", "another_package-");
    let (expected, observed) = compared(&expected, &different, &source()).unwrap();
    assert_ne!(expected, observed);
}

#[test]
fn rust_library_hashes_do_not_replace_the_library_module_path() {
    let actual = "error: denied\n --> /rustc/0123456789abcdef0123456789abcdef01234567/library/core/src/marker.rs:99:2\n99 | sealed();";
    let expected = "error: denied\n --> $RUST/core/src/marker.rs\n   | sealed();";
    let (expected, observed) = compared(expected, actual, &source()).unwrap();
    assert_eq!(expected, observed);
    let changed = actual.replace("core/src/marker.rs", "alloc/src/marker.rs");
    let (expected, observed) = compared(&expected, &changed, &source()).unwrap();
    assert_ne!(expected, observed);
}

#[test]
fn unknown_absolute_location_classes_are_refused() {
    let text = "error: denied\n --> /unrecognized/tree/contracts.rs:1:2";
    assert!(compared(text, text, &source()).is_err());
}

#[test]
fn source_text_and_labels_keep_all_spacing_after_the_renderer_gutter() {
    let actual = format!("error: denied\n --> {}:919:4\n919 |   pub fn sealed() {{}}\n    |      ^^^^^^ defined here", workspace().join("crates/worth-foundational/src/contracts.rs").display());
    let expected = "error: denied\n --> src/contracts.rs\n  |   pub fn sealed() {}\n  |      ^^^^^^ defined here";
    let (expected, observed) = compared(expected, &actual, &source()).unwrap();
    assert_eq!(expected, observed);
    for changed in [
        actual.replace("|   pub", "|  pub"),
        actual.replace("^^^^^^", "^^^^^"),
        actual.replace("defined here", "private"),
    ] {
        let (expected, observed) = compared(&expected, &changed, &source()).unwrap();
        assert_ne!(expected, observed);
    }
}

#[test]
fn rendered_source_containing_a_location_arrow_is_preserved_verbatim() {
    let text = format!("9 | // --> {}:4:2", source().display());
    assert_eq!(normalize(&text, &source()), text);
}

#[test]
fn existing_other_implementor_placeholder_preserves_all_eight_asserted_names() {
    let heading = "  = help: the following other types implement trait `Sealed`:";
    let entries = (0..8)
        .map(|index| format!("            Named{index}\n"))
        .collect::<String>();
    let expected = format!("{heading}\n{entries}          and $N others");
    let verbose = format!("{heading}\n{entries}            ExtraA\n            ExtraB");
    let (normalized, observed) = compared(&expected, &verbose, &source()).unwrap();
    assert_eq!(normalized, observed);
    let numeric = format!("{heading}\n{entries}          and 91 others");
    let (normalized, observed) = compared(&expected, &numeric, &source()).unwrap();
    assert_eq!(normalized, observed);
    for changed in [
        verbose.replace("Named7", "WrongName"),
        verbose.replace("trait `Sealed`", "trait `Other`"),
        format!("{heading}\n{entries}"),
    ] {
        let (normalized, observed) = compared(&expected, &changed, &source()).unwrap();
        assert_ne!(normalized, observed);
    }
}

#[test]
fn an_explicit_other_implementor_count_or_name_is_not_masked() {
    let heading = "  = help: the following other types implement trait `Sealed`:";
    let entries = (0..8)
        .map(|index| format!("            Named{index}\n"))
        .collect::<String>();
    let expected = format!("{heading}\n{entries}          and 91 others");
    let observed = expected.replace("91 others", "92 others");
    let (normalized, observed) = compared(&expected, &observed, &source()).unwrap();
    assert_ne!(normalized, observed);
}

#[test]
fn an_existing_constructor_count_placeholder_masks_only_the_numeric_tail() {
    let expected = "note: consider these associated functions:\n      Target::first\n      Target::second\n      and $N others";
    let actual = expected.replace("$N", "19");
    let (expected, observed) = compared(expected, &actual, &source()).unwrap();
    assert_eq!(expected, observed);
    let changed = actual.replace("Target::second", "Target::other");
    let (expected, observed) = compared(&expected, &changed, &source()).unwrap();
    assert_ne!(expected, observed);
}
