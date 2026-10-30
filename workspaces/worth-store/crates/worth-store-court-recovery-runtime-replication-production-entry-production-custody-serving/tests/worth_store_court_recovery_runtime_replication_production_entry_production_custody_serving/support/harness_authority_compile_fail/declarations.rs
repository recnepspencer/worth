pub(crate) use std::path::Path;

pub(crate) use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

pub(crate) const PHYSICAL_FIXTURES: &[(&str, &[&str])] = &[
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

pub(crate) const PRODUCTION_FIXTURES: &[(&str, &[&str])] = &[(
    "platform_physical_runtime_denial_receipt_requires_certification_authority.rs",
    &["from_append_hidden_scan_denial", "not found"],
)];

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("test-support crate lives under the Store workspace")
}
