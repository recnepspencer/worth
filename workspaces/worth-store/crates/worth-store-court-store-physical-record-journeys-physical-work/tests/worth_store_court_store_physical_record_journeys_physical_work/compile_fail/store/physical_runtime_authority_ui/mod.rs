#[allow(unused_imports)]
use crate::physical_runtime_authority_ui::*;
#[test]
fn external_consumers_cannot_forge_or_duplicate_runtime_authority() {
    crate::bounded_physical_record_access_examples::run_configuration_examples();

    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/runtime_duplication_and_reconstruction_are_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/internal_composition_construction_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/internal_runtime_topology_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/non_authority_values_cannot_admit.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs")).expect("prebuilt compiler denial");

    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/frame_view_cannot_outlive_lease.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/lower_clean_authority_is_required.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_receipt_construction_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_work_identity_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_work_progression_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/untyped_physical_work_basis_is_rejected.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/borrowed_physical_work_submission_is_rejected.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/legacy_mutation_and_writeback_routes_are_absent.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_residency_policy_cannot_enter_open.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/admitted_residency_policy_is_sealed.rs")).expect("prebuilt compiler denial");
    durability_policy_cases();
    record_chunk_view_cases();
}

#[test]
fn resident_admission_contracts_are_compile_sealed() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_lease_cannot_enter_resident_decoder.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/resident_admission_internals_are_sealed.rs")).expect("prebuilt compiler denial");
}

#[test]
fn recovery_handoff_is_compile_sealed() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_handoff_constructor_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_handoff_is_linear.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_handoff_rejects_report_conversion.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_operation_fact_cannot_mint_handoff.rs")).expect("prebuilt compiler denial");
}

#[test]
fn progression_requires_exact_predecessor_authority() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wal_durable_authority_requires_completed_barrier.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_data_progression_is_sealed.rs")).expect("prebuilt compiler denial");
}

#[test]
fn checkpoint_authority_is_compile_bound() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/checkpoint_capture_authority_is_sealed.rs")).expect("prebuilt compiler denial");
}

#[test]
fn root_publication_ownership_is_compile_bound() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/root_publication_plans_are_linear.rs")).expect("prebuilt compiler denial");
}

#[test]
fn ordinary_mutation_outcomes_are_compile_bound() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/ordinary_mutation_phase_driving_is_absent.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/noncompleted_mutation_cannot_acknowledge.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/mutation_evidence_cannot_reenter_authority.rs")).expect("prebuilt compiler denial");
}

#[test]
fn blob_ingest_keeps_store_allocation_and_runtime_authority_bound() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/blob_recovery_allocation_cannot_enter_ingest.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/blob_ingest_cannot_escape_runtime.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/runtime_cannot_close_with_blob_ingest.rs")).expect("prebuilt compiler denial");
}

#[test]
fn blob_relocation_requires_store_selected_hold() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/stable_read_receipt_cannot_relocate_blob.rs")).expect("prebuilt compiler denial");
}

#[test]
fn terminal_head_retirement_authority_is_owner_issued() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/terminal_head_retirement_authority_is_sealed.rs")).expect("prebuilt compiler denial");
}
