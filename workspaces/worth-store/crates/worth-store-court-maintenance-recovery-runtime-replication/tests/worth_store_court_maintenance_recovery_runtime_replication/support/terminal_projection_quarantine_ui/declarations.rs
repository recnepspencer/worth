pub(crate) use std::path::Path;

pub(crate) use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[
    (
        "store_identity_does_not_implement_display.rs",
        &["doesn't implement `std::fmt::Display`"],
    ),
    (
        "store_identity_has_no_neutral_string_accessor.rs",
        &["no method named `as_str`"],
    ),
    (
        "store_locator_has_no_neutral_string_accessor.rs",
        &["no method named `as_str`"],
    ),
    (
        "terminal_json_projection_cannot_satisfy_boundary_fact.rs",
        &["expected `StoreAspectBoundaryFact`"],
    ),
    (
        "terminal_json_projection_document_is_not_public.rs",
        &["no method named `terminal_projection_document`"],
    ),
    (
        "terminal_json_projection_has_no_public_document_constructor.rs",
        &["associated function `from_terminal_projection_document` is private"],
    ),
    (
        "terminal_json_document_checksum_cannot_satisfy_digest_authority.rs",
        &["found `StoreTerminalDocumentChecksum`"],
    ),
    (
        "terminal_projection_text_cannot_satisfy_identity.rs",
        &["expected `StoreAspectIdentity`"],
    ),
];

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}
