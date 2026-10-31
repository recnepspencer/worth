pub(crate) fn durability_policy_cases() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/incomplete_durability_policy_cannot_admit.rs"),
        "E0599",
        &["no method named `admit`", "PhysicalDurabilityDeclarationBuilder<", "::GroupConfigured,", "::WalMissing,", "::IdempotencyMissing,", "::CheckpointMissing>"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_backend_profile_cannot_admit_durability.rs"),
        "E0308",
        &["expected `PhysicalDurabilityAdmissionBasis`, found `BackendTargetProfile`", "arguments to this method are incorrect"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_durability_basis_cannot_be_duplicated.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_uncoded_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/admitted_durability_policy_is_sealed.rs"),
        "cannot construct",
        &["`AdmittedPhysicalDurabilityPolicy`", "private fields"],
    ).expect("prebuilt uncoded semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/durability_policy_cannot_be_omitted_from_open.rs"),
        "E0061",
        &["`AdmittedPhysicalDurabilityPolicy` is missing"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_mutation_preparation_authority_is_sealed.rs")).expect("prebuilt compiler denial");
}

pub(crate) fn record_chunk_view_cases() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/root_observation_cannot_mint_reader.rs"),
        "E0277",
        &["PhysicalRecordReader", "From<PhysicalProtectedRootObservation>"],
    ).expect("prebuilt semantic compiler denial");

    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_cannot_escape_session.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_blocks_session_progress.rs"),
        "E0499",
        &["cannot borrow `session` as mutable more than once at a time", "first mutable borrow occurs here", "second mutable borrow occurs here", "first borrow later used here"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_blocks_session_drop.rs"),
        "E0505",
        &["cannot move out of `session` because it is borrowed", "borrow of `session` occurs here", "move out of `session` occurs here", "borrow later used here"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_bytes_retain_session_borrow.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_construction_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_exposes_no_pool_authority.rs"),
        "E0614",
        &["PhysicalRecordChunkView"],
    )
        .expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/opened_physical_record_alias_is_absent.rs"),
        "E0432",
        &["worth_store::physical_runtime::OpenedPhysicalRecord"],
    ).expect("prebuilt semantic compiler denial");
}
