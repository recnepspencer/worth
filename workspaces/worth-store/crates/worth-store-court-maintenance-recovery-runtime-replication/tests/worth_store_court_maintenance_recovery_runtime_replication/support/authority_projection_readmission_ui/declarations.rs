pub(crate) use std::path::Path;

pub(crate) use {
    worth_store_test_support::compiler_boundary::cargo_dependency_manifest,
    worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite,
};

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[
    (
        "digest_string_cannot_satisfy_current_authority.rs",
        &["StoreCurrentAuthorityWitness", "String"],
    ),
    (
        "derived_evidence_cannot_satisfy_current_authority.rs",
        &[
            "StoreCurrentAuthorityWitness",
            "StoreDerivedAuthorityEvidence",
        ],
    ),
    (
        "external_token_cannot_satisfy_current_authority.rs",
        &[
            "StoreCurrentAuthorityWitness",
            "StoreExternalAuthorityToken",
        ],
    ),
    (
        "filename_cannot_satisfy_current_authority.rs",
        &["StoreCurrentAuthorityWitness", "StoreAuthorityFilename"],
    ),
    (
        "retained_evidence_cannot_satisfy_current_physical_authority.rs",
        &[
            "StoreCurrentPhysicalAuthorityWitness",
            "StoreRetainedAuthorityEvidence",
        ],
    ),
    (
        "stable_id_cannot_construct_canonical_authority_record.rs",
        &["CanonicalAuthorityRecord", "new"],
    ),
    (
        "stable_id_cannot_satisfy_canonical_authority_witness.rs",
        &["StoreCurrentAuthorityWitness", "StableArtifactId"],
    ),
    (
        "terminal_projection_text_cannot_satisfy_current_authority.rs",
        &[
            "StoreCurrentAuthorityWitness",
            "StoreTerminalProjectionText",
        ],
    ),
];

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-store-certification"
    ))
    .ancestors()
    .nth(2)
    .expect("certification crate lives under the Store workspace")
}
