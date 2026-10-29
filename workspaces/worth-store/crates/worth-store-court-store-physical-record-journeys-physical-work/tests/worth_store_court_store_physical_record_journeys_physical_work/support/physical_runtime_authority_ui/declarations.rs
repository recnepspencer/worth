

pub(crate) fn durability_policy_cases(cases: &trybuild::TestCases) {
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_wal_append_examples.rs"));
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/incomplete_durability_policy_cannot_admit.rs"),
    );
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_backend_profile_cannot_admit_durability.rs"),
    );
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_durability_basis_cannot_be_duplicated.rs"),
    );
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/admitted_durability_policy_is_sealed.rs"));
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/durability_policy_cannot_be_omitted_from_open.rs"),
    );
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_mutation_preparation_authority_is_sealed.rs"),
    );
}

pub(crate) fn record_chunk_view_cases(cases: &trybuild::TestCases) {
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/protected_record_reads_supported.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/root_observation_cannot_mint_reader.rs"));
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/support/bounded_physical_record_access_examples/declarations.rs"));
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_views_supported.rs"));
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_cannot_escape_session.rs"),
    );
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_blocks_session_progress.rs"),
    );
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_blocks_session_drop.rs"));
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_bytes_retain_session_borrow.rs"),
    );
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_construction_is_sealed.rs"),
    );
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_exposes_no_pool_authority.rs"),
    );
    cases
        .compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/opened_physical_record_alias_is_absent.rs"));
}