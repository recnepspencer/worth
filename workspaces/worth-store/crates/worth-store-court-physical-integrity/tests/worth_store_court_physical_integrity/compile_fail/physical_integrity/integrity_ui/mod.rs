#[test]
fn physical_integrity_construction_boundaries() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/proof_construction.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/root_tree_proof_construction.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/proof_escape.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/projection_cannot_open_decoder.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/record_cannot_open_decoder.rs"),
        "E0308",
        &["&[u8]", "PhysicalIntegrityValidationRecord"],
    )
    .expect("validation records cannot substitute for decoder bytes");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/untrusted_page_extent_cannot_project.rs"),
        "E0308",
        &["UntrustedPhysicalArtifact", "IntegrityValidatedPageFrame"],
    )
    .expect("untrusted bytes cannot substitute for validated page authority");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/untrusted_page_extent_cannot_project.rs"),
        "E0308",
        &["UntrustedPhysicalArtifact", "IntegrityValidatedExtentChunkFrame"],
    )
    .expect("untrusted bytes cannot substitute for validated extent authority");
}
