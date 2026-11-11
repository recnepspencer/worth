use super::merge_proof_journey::*;
use worth_relational::facade::merge::RelationalMergeProofPacketAdmissionPosture;
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn retained_merge_proof_packet_preserves_exact_authority_truth() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge");
    let proof_packet = runtime
        .merge()
        .retain_merge_proof_packet_from_prepared_execution(&prepared);

    assert_eq!(proof_packet.request(), prepared.request());
    assert_eq!(
        proof_packet.branch_basis(),
        &prepared.artifact().branch_basis
    );
    let correspondence_witness = runtime
        .merge()
        .retain_merge_correspondence_witness_from_prepared_execution(&prepared);
    assert_eq!(
        proof_packet.correspondence_witness_digest(),
        correspondence_witness.witness_digest()
    );
    let schema_witness = runtime
        .merge()
        .retain_schema_reconciliation_witness_from_prepared_execution(&prepared);
    assert_eq!(
        proof_packet.schema_reconciliation_witness_digest(),
        schema_witness.witness_digest()
    );
    let expected_surface = prepared
        .artifact()
        .lowered_plan
        .records
        .iter()
        .map(|row| (&row.record, row.target_record.as_ref()))
        .collect::<Vec<_>>();
    let actual_surface = proof_packet
        .admitted_merge_surface()
        .iter()
        .map(|row| (row.record(), row.target_record()))
        .collect::<Vec<_>>();
    assert_eq!(actual_surface, expected_surface);
    assert_eq!(
        proof_packet.admission_posture(),
        RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted
    );
    assert!(!proof_packet.planning_digest().is_empty());
    assert!(!proof_packet
        .foundational_request_lowering_digest()
        .is_empty());
    assert!(!proof_packet.admitted_merge_surface_digest().is_empty());
    assert!(!proof_packet.packet_digest().is_empty());
    let outcome = runtime.execute_prepared_merge(execution, prepared).unwrap();
    assert_eq!(
        proof_packet.execution_digest(),
        outcome.execution_summary.execution_digest
    );
}
#[test]
fn published_merge_outcome_retains_proof_packet_as_authority() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    let outcome = execute_merge(execution, &runtime);
    let summary_packet = outcome.execution_summary.proof_packet().clone();
    let live_authority =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);

    assert_eq!(
        summary_packet,
        outcome.execution_summary.proof_packet.clone()
    );
    assert_eq!(
        summary_packet,
        live_authority.execution_summary.proof_packet.clone()
    );
    assert_eq!(
        outcome
            .execution_summary
            .correspondence_witness
            .witness_digest(),
        summary_packet.correspondence_witness_digest()
    );
    assert_eq!(summary_packet.request(), &outcome.execution_summary.request);
    assert_eq!(
        summary_packet.branch_basis(),
        &outcome.execution_summary.branch_basis
    );
    assert_eq!(
        summary_packet.execution_digest(),
        outcome.execution_summary.execution_digest
    );
    assert_eq!(
        outcome.execution_summary.target_head_commit_id,
        summary_packet.branch_basis().target_head().commit_id
    );
    assert_eq!(
        outcome.execution_summary.source_head_commit_id,
        summary_packet.branch_basis().source_head().commit_id
    );
    assert_eq!(
        outcome.execution_summary.merge_base_commit_id,
        summary_packet
            .branch_basis()
            .merge_base()
            .commit()
            .commit_id
    );
    runtime
        .inspect_what_happened()
        .prepare_merge_support_inspection_witness(&outcome.execution_summary)
        .expect("consistent summary authority");
    runtime
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&live_authority)
        .expect("consistent published authority");
}
#[test]
fn merge_proof_packet_survives_publication_and_recovery_without_drift() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    let outcome = execute_merge(execution, &runtime);
    let live_packet = outcome.execution_summary.proof_packet.clone();
    let live_authority =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);
    let (_recovery, recovered) = checkpoint_and_recover(execution, &runtime, &medium);
    let recovered_authority =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id);

    assert_eq!(live_packet, live_authority.execution_summary.proof_packet);
    assert_eq!(
        live_packet,
        recovered_authority.execution_summary.proof_packet
    );
    assert_eq!(
        live_packet.packet_digest(),
        recovered_authority
            .execution_summary
            .proof_packet
            .packet_digest()
    );
}
