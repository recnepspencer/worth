#[allow(unused_imports)]
use crate::physical_runtime_authority_ui::*;
mod close_task;
#[test]
fn external_consumers_cannot_forge_or_duplicate_runtime_authority() {
    crate::bounded_physical_record_access_examples::run_configuration_examples();

    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/runtime_duplication_and_reconstruction_are_sealed.rs"),
        "E0599",
        &["clone", "AdmittedPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/runtime_duplication_and_reconstruction_are_sealed.rs"),
        "E0308",
        &["PhysicalRuntimeAdmission", "RuntimeIdentity"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/internal_composition_construction_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/internal_runtime_topology_is_sealed.rs"),
        "E0603",
        &["lifecycle"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/internal_runtime_topology_is_sealed.rs"),
        "E0603",
        &["root_admission"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/non_authority_values_cannot_admit.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["append_physical_record", "AdmittedPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["open_physical_media", "AdmittedPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["media_observation", "AdmittedPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["lookup_owner", "AdmittedPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["append_physical_record", "ObservationHandle"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["publish_physical_changes", "ObservationHandle"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["recover_physical_store", "ObservationHandle"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["close", "ObservationHandle"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["raw_physical_owner", "ObservationHandle"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["observe", "ClosedRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0599",
        &["observe", "AbortedRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wrong_phase_and_physical_operations_are_absent.rs"),
        "E0382",
        &["AdmittedPhysicalRuntime::close", "moves `runtime`"],
    ).expect("prebuilt semantic compiler denial");

    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/frame_view_cannot_outlive_lease.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/lower_clean_authority_is_required.rs"),
        "E0061",
        &["FrameWritebackCleanAuthority", "argument #1 of type `&FrameWritebackCleanAuthority` is missing"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/lower_clean_authority_is_required.rs"),
        "E0061",
        &["CandidateFrameCleanAuthority", "argument #1 of type `&CandidateFrameCleanAuthority` is missing"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/lower_clean_authority_is_required.rs"),
        "E0308",
        &["FrameWritebackCleanAuthority", "CandidateFrameCleanAuthority", "expected `&FrameWritebackCleanAuthority`, found `&CandidateFrameCleanAuthority`"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/lower_clean_authority_is_required.rs"),
        "E0308",
        &["FrameWritebackCleanAuthority", "CandidateFrameCleanAuthority", "expected `&CandidateFrameCleanAuthority`, found `&FrameWritebackCleanAuthority`"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_receipt_construction_is_sealed.rs"),
        "E0451",
        &["CompletedArtifactRangeWrite", "`owner`", "`store`", "`coordinate`", "`payload_digest`", "`completed_bytes`", "`operation`", "`durability`", "private"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_work_identity_is_sealed.rs"),
        "E0451",
        &["PhysicalWorkIdentity", "`store`", "`runtime`", "`generation`", "`operation`", "private"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_work_progression_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/untyped_physical_work_basis_is_rejected.rs"),
        "E0308",
        &["expected `PhysicalWorkSemanticBasis`", "found `AspectValue`"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/untyped_physical_work_basis_is_rejected.rs"),
        "E0308",
        &["expected `PhysicalWorkSemanticBasis`", "found `Aspect`"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/untyped_physical_work_basis_is_rejected.rs"),
        "E0308",
        &["expected `PhysicalWorkSemanticBasis`", "found `Value`"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/borrowed_physical_work_submission_is_rejected.rs"),
        "E0308",
        &["expected `PhysicalReadWorkRequest`", "found `&PhysicalReadWorkRequest`"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/legacy_mutation_and_writeback_routes_are_absent.rs"),
        "E0599",
        &["records_mut", "ServingPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/legacy_mutation_and_writeback_routes_are_absent.rs"),
        "E0599",
        &["execute_scheduled_writeback", "ServingPhysicalRuntime"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs"),
        "E0599",
        &["new", "AdmittedDirtyFrame"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs"),
        "E0616",
        &["frame", "AdmittedDirtyFrame"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs"),
        "E0616",
        &["receipt", "PreparedPhysicalWriteback"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs"),
        "E0616",
        &["ready", "ReadyPhysicalWriteback"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs"),
        "E0616",
        &["work", "AdmittedPhysicalWriteback"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/residency_writeback_internals_are_sealed.rs"),
        "E0616",
        &["dirty", "PhysicalWritebackTransitionFailure"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_residency_policy_cannot_enter_open.rs"),
        "E0308",
        &["AdmittedPhysicalRecordResidencyPolicy", "PhysicalRecordResidencyPolicyBuilder"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/admitted_residency_policy_is_sealed.rs")).expect("prebuilt compiler denial");
    durability_policy_cases();
    record_chunk_view_cases();
}

#[test]
fn resident_admission_contracts_are_compile_sealed() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/raw_lease_cannot_enter_resident_decoder.rs"),
        "E0599",
        &["with_owner_decoder", "PhysicalFrameLease"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/resident_admission_internals_are_sealed.rs"),
        "E0432",
        &["worth_store::physical_runtime::IntegrityAdmittedResidentPage", "worth_store::physical_runtime::ResidentAdmissionContext", "worth_store::physical_runtime::ResidentIntegrityRecordBinding"],
    ).expect("prebuilt semantic compiler denial");
}

#[test]
fn recovery_handoff_is_compile_sealed() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_handoff_constructor_is_sealed.rs"),
        "E0599",
        &["`new`", "PhysicalDurabilityRecoveryHandoff"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_handoff_rejects_report_conversion.rs"),
        "E0277",
        &["PhysicalDurabilityRecoveryHandoff", "From<PhysicalDurabilityPerformanceSummary>"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/c8_recovery_operation_fact_cannot_mint_handoff.rs"),
        "E0277",
        &["PhysicalDurabilityRecoveryHandoff", "From<PhysicalRecoveryOperationFact>"],
    ).expect("prebuilt semantic compiler denial");
}

#[test]
fn progression_requires_exact_predecessor_authority() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/wal_durable_authority_requires_completed_barrier.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_data_progression_is_sealed.rs")).expect("prebuilt compiler denial");
}

#[test]
fn checkpoint_authority_is_compile_bound() {
    worth_testkit_compiler::certification_uncoded_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/checkpoint_capture_authority_is_sealed.rs"),
        "cannot construct",
        &["CompletedPhysicalCheckpoint", "private fields"],
    ).expect("prebuilt uncoded semantic compiler denial");
}

#[test]
fn root_publication_ownership_is_compile_bound() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/root_publication_plans_are_linear.rs"),
        "E0277",
        &["Clone", "RootPublicationPlanningMembers"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/root_publication_plans_are_linear.rs"),
        "E0277",
        &["Clone", "RootPublicationCandidatePlan"],
    ).expect("prebuilt semantic compiler denial");
}

#[test]
fn ordinary_mutation_outcomes_are_compile_bound() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/ordinary_mutation_phase_driving_is_absent.rs"),
        "E0624",
        &["append_prepared_wal_group"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/noncompleted_mutation_cannot_acknowledge.rs"),
        "E0599",
        &["into_acknowledgment", "ProvenNoEffectPhysicalMutation"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/noncompleted_mutation_cannot_acknowledge.rs"),
        "E0599",
        &["into_acknowledgment", "IndeterminatePhysicalMutation"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/mutation_evidence_cannot_reenter_authority.rs"),
        "E0599",
        &["into_acknowledgment", "PhysicalMutationExecutedBoundaryEvidence"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/mutation_evidence_cannot_reenter_authority.rs"),
        "E0599",
        &["into_acknowledgment", "PhysicalMutationPerformanceEvidence"],
    ).expect("prebuilt semantic compiler denial");
}

#[test]
fn blob_ingest_keeps_store_allocation_and_runtime_authority_bound() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/blob_recovery_allocation_cannot_enter_ingest.rs"),
        "E0308",
        &["expected `BlobIngestDeclaration`", "found `RecoveryPhysicalAllocation<"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/blob_ingest_cannot_escape_runtime.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/runtime_cannot_close_with_blob_ingest.rs")).expect("prebuilt compiler denial");
}

#[test]
fn blob_relocation_requires_store_selected_hold() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/stable_read_receipt_cannot_relocate_blob.rs"),
        "E0308",
        &["BlobMovementReadHold", "StablePhysicalReadReceipt", "because of return type"],
    ).expect("prebuilt semantic compiler denial");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/stable_read_receipt_cannot_relocate_blob.rs"),
        "E0308",
        &["BlobMovementReadHold", "StablePhysicalReadReceipt", "arguments to this method are incorrect"],
    ).expect("prebuilt semantic compiler denial");
}

#[test]
fn terminal_head_retirement_authority_is_owner_issued() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/terminal_head_retirement_authority_is_sealed.rs"),
        "E0432",
        &["worth_store::physical_runtime::CheckpointAttestedTerminalHead", "worth_store::physical_runtime::TerminalHeadIdentityNonReissue", "worth_store::physical_runtime::TerminalHeadNoReaderOrRecoveryHold", "worth_store::physical_runtime::TerminalHeadNoRetryClaim", "worth_store::physical_runtime::TerminalHeadPublicationExcluded", "worth_store::physical_runtime::TerminalHeadRetirementAuthority"],
    ).expect("prebuilt semantic compiler denial");
}

mod owned_publication;

mod owned_ordered_journal;
