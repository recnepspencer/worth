pub(crate) use std::path::Path;

pub(crate) use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[
    ("copied_row_cannot_publish.rs", &["with_row", "private"]),
    (
        "proof_authority_constructor_is_private.rs",
        &["private", "_private"],
    ),
    (
        "proof_authority_factory_is_private.rs",
        &["from_executed_store_evidence", "private"],
    ),
    (
        "row_proof_constructor_is_private.rs",
        &["from_admitted_backend_witness_with_proof", "private"],
    ),
];

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}
