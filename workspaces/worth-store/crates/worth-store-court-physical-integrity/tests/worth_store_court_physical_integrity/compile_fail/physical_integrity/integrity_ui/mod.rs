#[test]
fn physical_integrity_construction_boundaries() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedCheckpointDirtyBasis<", "found `IntegrityValidatedCheckpointBinding<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedCurrentRootSelector<", "found `IntegrityValidatedPreviousRootSelector<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedCurrentRootSelector<", "found `IntegrityValidatedPhysicalWorkObligation<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedPhysicalWorkObligation<", "found `IntegrityValidatedCurrentRootSelector<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedPageFrame<", "found `IntegrityValidatedCurrentRootSelector<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedCurrentRootSelector<", "found `IntegrityValidatedWalFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedExtentManifest<", "found `IntegrityValidatedExtentChunkFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedRootRoutingBlock<", "found `IntegrityValidatedSegmentMembershipBlock<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"),
        "E0308",
        &["expected `IntegrityValidatedRootRoutingBlock<", "found `IntegrityValidatedBootstrapCatalog<"],
    )
    .expect("fixture retains its intended authority type and role denial");
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
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0599",
        &["`with_scope`", "struct `IntegrityValidatedRootManifest<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0599",
        &["`with_scope`", "struct `IntegrityValidatedPhysicalWorkObligation<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0599",
        &["`with_wal_lsn_range`", "struct `PhysicalArtifactScope`"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0599",
        &["`with_scope`", "struct `IntegrityValidatedExtentManifest<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0599",
        &["`with_scope`", "struct `IntegrityValidatedExtentChunkFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0599",
        &["`with_scope`", "struct `IntegrityValidatedCheckpointDirtyBasis<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"),
        "E0308",
        &["expected `PhysicalCheckpointIdentity`", "found `CheckpointStreamHeaderScopeIdentity`"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedCheckpointStreamHeader<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedCheckpointDirtyBasis<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedCheckpointBindingCompaction<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedCheckpointBinding<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedCheckpointFooter<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedCurrentRootSelector<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedPreviousRootSelector<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedRootManifest<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedPhysicalWorkObligation<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedPageFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedWalFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedExtentManifest<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0308",
        &["expected `&[u8]`", "found `IntegrityValidatedExtentChunkFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "//tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"),
        "E0599",
        &["`chunk_bytes`", "struct `IntegrityValidatedExtentChunkFrame<"],
    )
    .expect("fixture retains its intended authority type and role denial");
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
