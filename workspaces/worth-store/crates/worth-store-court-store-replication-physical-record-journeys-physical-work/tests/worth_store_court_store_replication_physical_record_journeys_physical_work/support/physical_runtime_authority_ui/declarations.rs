pub(crate) fn durability_policy_cases() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/incomplete_durability_policy_cannot_admit.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_backend_profile_cannot_admit_durability.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_durability_basis_cannot_be_duplicated.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/admitted_durability_policy_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/durability_policy_cannot_be_omitted_from_open.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_mutation_preparation_authority_is_sealed.rs")).expect("prebuilt compiler denial");
}

pub(crate) fn record_chunk_view_cases() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/root_observation_cannot_mint_reader.rs")).expect("prebuilt compiler denial");

    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_cannot_escape_session.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_blocks_session_progress.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_blocks_session_drop.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_bytes_retain_session_borrow.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_construction_is_sealed.rs")).expect("prebuilt compiler denial");
    use worth_store_test_support::compiler_boundary::{
        read_ui_proof_suite, ExpectedCompilerDenial, UiCompilerEnvironment, UiFixtureDeclaration,
        UiProofSuiteDeclaration,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let suite = UiProofSuiteDeclaration::new(
        "record-chunk-view-pool-authority",
        UiCompilerEnvironment::Certification,
        vec![UiFixtureDeclaration::new(
            "record_chunk_view_exposes_no_pool_authority",
            root.join("tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_exposes_no_pool_authority.rs"),
            ExpectedCompilerDenial::new(["E0614"], ["PhysicalRecordChunkView"]).unwrap(),
        ).unwrap()],
    ).unwrap();
    read_ui_proof_suite(root.ancestors().nth(2).unwrap(), &suite)
        .expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/opened_physical_record_alias_is_absent.rs")).expect("prebuilt compiler denial");
}
