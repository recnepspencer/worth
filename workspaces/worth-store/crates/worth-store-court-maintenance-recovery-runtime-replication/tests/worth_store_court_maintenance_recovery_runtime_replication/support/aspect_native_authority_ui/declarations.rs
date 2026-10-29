pub(crate) use {std::path::Path};

pub(crate) use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[
    (
        "local_diagnostic_payload_cannot_satisfy_store_evidence.rs",
        &[
            "StoreDiagnosticSupportReportEvidence",
            "LocalDiagnosticPayload",
        ],
    ),
    (
        "local_performance_claim_cannot_satisfy_store_evidence.rs",
        &["StorePerformanceReceiptEvidence", "LocalPerformanceClaim"],
    ),
    (
        "raw_aspect_value_cannot_satisfy_boundary_fact.rs",
        &["StoreAspectBoundaryFact", "AspectValue"],
    ),
    (
        "raw_string_cannot_satisfy_store_identity.rs",
        &["StoreAspectIdentity", "String"],
    ),
    (
        "raw_struct_cannot_satisfy_authority_input.rs",
        &["StoreAspectAuthorityInput", "StructAspectValue"],
    ),
    (
        "terminal_projection_text_cannot_satisfy_locator.rs",
        &["StoreAspectBoundaryLocator", "String"],
    ),
];

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-store-certification"))
        .ancestors()
        .nth(2)
        .expect("certification crate lives under the Store workspace")
}