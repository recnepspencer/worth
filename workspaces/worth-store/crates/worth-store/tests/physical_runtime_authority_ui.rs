#[path = "physical_runtime_authority/bounded_physical_record_access_examples.rs"]
#[allow(
    dead_code,
    reason = "the same file is also a standalone trybuild binary"
)]
mod bounded_physical_record_access_examples;

#[test]
fn external_consumers_cannot_forge_or_duplicate_runtime_authority() {
    bounded_physical_record_access_examples::run_configuration_examples();
    let cases = trybuild::TestCases::new();
    cases.pass("tests/physical_runtime_authority/supported_admission.rs");
    cases.pass("tests/physical_runtime_authority/supported_physical_work.rs");
    cases.pass("tests/physical_runtime_authority/admitted_residency_policy_supported.rs");
    cases.pass("tests/physical_runtime_authority/responsibility_named_store_facade_supported.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/runtime_duplication_and_reconstruction_are_sealed.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/internal_composition_construction_is_sealed.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/internal_runtime_topology_is_sealed.rs");
    cases.compile_fail("tests/physical_runtime_authority/non_authority_values_cannot_admit.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs",
    );
    cases.pass(
        "tests/physical_runtime_authority/independent_scan_and_mutation_capabilities_compile.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/frame_view_cannot_outlive_lease.rs");
    cases.compile_fail("tests/physical_runtime_authority/lower_clean_authority_is_required.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/physical_receipt_construction_is_sealed.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/physical_work_identity_is_sealed.rs");
    cases.compile_fail("tests/physical_runtime_authority/physical_work_progression_is_sealed.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/untyped_physical_work_basis_is_rejected.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/borrowed_physical_work_submission_is_rejected.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/legacy_mutation_and_writeback_routes_are_absent.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/residency_writeback_internals_are_sealed.rs",
    );
    cases
        .compile_fail("tests/physical_runtime_authority/raw_residency_policy_cannot_enter_open.rs");
    cases.compile_fail("tests/physical_runtime_authority/admitted_residency_policy_is_sealed.rs");
    durability_policy_cases(&cases);
    record_chunk_view_cases(&cases);
}

#[test]
fn c9_resident_admission_contracts_are_compile_sealed() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/physical_runtime_authority/raw_lease_can_only_observe_descriptive_record.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/raw_lease_cannot_enter_resident_decoder.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/resident_admission_internals_are_sealed.rs",
    );
}

fn durability_policy_cases(cases: &trybuild::TestCases) {
    cases.pass("tests/physical_runtime_authority/physical_wal_append_examples.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/incomplete_durability_policy_cannot_admit.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/raw_backend_profile_cannot_admit_durability.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/physical_durability_basis_cannot_be_duplicated.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/admitted_durability_policy_is_sealed.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/durability_policy_cannot_be_omitted_from_open.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/physical_mutation_preparation_authority_is_sealed.rs",
    );
}

#[test]
fn phase_ten_c8_recovery_handoff_is_compile_sealed() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(
        "tests/physical_runtime_authority/c8_recovery_handoff_constructor_is_sealed.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/c8_recovery_handoff_is_linear.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/c8_recovery_handoff_rejects_report_conversion.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/c8_recovery_operation_fact_cannot_mint_handoff.rs",
    );
}

#[test]
fn phase_four_progression_requires_exact_predecessor_authority() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/physical_runtime_authority/physical_data_progression_examples.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/wal_durable_authority_requires_completed_barrier.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/physical_data_progression_is_sealed.rs");
}

#[test]
fn phase_six_checkpoint_authority_is_compile_bound() {
    let cases = trybuild::TestCases::new();
    cases
        .compile_fail("tests/physical_runtime_authority/checkpoint_capture_authority_is_sealed.rs");
}

#[test]
fn phase_seven_root_publication_ownership_is_compile_bound() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/physical_runtime_authority/root_publication_plans_are_linear.rs");
}

#[test]
fn phase_eight_ordinary_mutation_outcomes_are_compile_bound() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/physical_runtime_authority/ordinary_mutation_outcomes_are_supported.rs");
    cases.pass("tests/physical_runtime_authority/physical_durability_guide_examples.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/ordinary_mutation_phase_driving_is_absent.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/noncompleted_mutation_cannot_acknowledge.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/mutation_evidence_cannot_reenter_authority.rs",
    );
}

fn record_chunk_view_cases(cases: &trybuild::TestCases) {
    cases.pass("tests/physical_runtime_authority/bounded_physical_record_access_examples.rs");
    cases.pass("tests/physical_runtime_authority/record_chunk_views_supported.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/record_chunk_view_cannot_escape_session.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/record_chunk_view_blocks_session_progress.rs",
    );
    cases.compile_fail("tests/physical_runtime_authority/record_chunk_view_blocks_session_drop.rs");
    cases.compile_fail(
        "tests/physical_runtime_authority/record_chunk_bytes_retain_session_borrow.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/record_chunk_view_construction_is_sealed.rs",
    );
    cases.compile_fail(
        "tests/physical_runtime_authority/record_chunk_view_exposes_no_pool_authority.rs",
    );
    cases
        .compile_fail("tests/physical_runtime_authority/opened_physical_record_alias_is_absent.rs");
}
