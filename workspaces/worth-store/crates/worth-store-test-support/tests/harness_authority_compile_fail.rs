#![cfg_attr(test, allow(clippy::disallowed_methods))]
use std::path::Path;

use worth_store_test_support::compiler_boundary::{
    cargo_dependency_manifest, run_cargo_ui_fixture_suite,
};

#[test]
fn courtroom_test_authority_cannot_satisfy_production_surfaces() {
    let root = store_workspace_root();
    let source = root.join("crates/worth-store-test-support/tests/ui/harness_authority");
    let physical = run_cargo_ui_fixture_suite(
        root,
        "test-support-harness-physical-authority",
        physical_environment(root),
        "physical-isolation-fixtures",
        "diagnostic-test",
        &source,
        PHYSICAL_FIXTURES,
    )
    .unwrap();
    let production = run_cargo_ui_fixture_suite(
        root,
        "test-support-harness-production-authority",
        production_environment(root),
        "production",
        "diagnostic-test",
        &source,
        PRODUCTION_FIXTURES,
    )
    .unwrap();
    assert_eq!(
        physical.fixtures.len() + production.fixtures.len(),
        PHYSICAL_FIXTURES.len() + PRODUCTION_FIXTURES.len()
    );
}

const PHYSICAL_FIXTURES: &[(&str, &[&str])] = &[
    (
        "harness_reference_is_not_physical_reference.rs",
        &["PhysicalReference", "HarnessPhysicalReference"],
    ),
    (
        "harness_reference_cannot_expose_private_reference_lane.rs",
        &["no method named `as_physical_reference`"],
    ),
    (
        "platform_physical_runtime_receipt_cannot_be_minted.rs",
        &["PlatformPhysicalModelReceipt", "private fields"],
    ),
];

const PRODUCTION_FIXTURES: &[(&str, &[&str])] = &[(
    "platform_physical_runtime_denial_receipt_requires_certification_authority.rs",
    &["from_append_hidden_scan_denial", "not found"],
)];

fn physical_environment(root: &Path) -> String {
    cargo_dependency_manifest(
        &[
            (
                "worth-store-contracts",
                root.join("crates/worth-store-contracts").as_path(),
                &[],
            ),
            (
                "worth-store-physical-format",
                root.join("crates/worth-store-physical-format").as_path(),
                &[],
            ),
            (
                "worth-store-test-support",
                root.join("crates/worth-store-test-support").as_path(),
                &["physical-isolation-fixtures"],
            ),
            (
                "worth-store-physical-certification",
                root.join("crates/worth-store-physical-certification")
                    .as_path(),
                &[],
            ),
        ],
        &[],
    )
}

fn production_environment(root: &Path) -> String {
    cargo_dependency_manifest(
        &[
            (
                "worth-store-contracts",
                root.join("crates/worth-store-contracts").as_path(),
                &[],
            ),
            (
                "worth-store-physical-format",
                root.join("crates/worth-store-physical-format").as_path(),
                &[],
            ),
        ],
        &[],
    )
}

fn store_workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("test-support crate lives under the Store workspace")
}
