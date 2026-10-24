use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionOutcome, MergeExecutionRequest, MergeIntent};
use crate::facade::runtime::RelationalRuntime;
use crate::merge::data::RelationalMergeAdmittedSurfaceRow;
use crate::tests::support::{
    checkpoint_and_recover_with, create_branch_from_main, create_entity,
    create_entity_outcome_on_branch, durable_runtime_with_test_schema,
};
use crate::transactions::data::PublishedMergeExecutionAuthority;

#[test]
fn retained_merge_proof_packet_preserves_exact_authority_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let proof_packet = runtime
        .merge()
        .retain_merge_proof_packet_from_prepared_execution(&prepared);

    assert_eq!(proof_packet.request(), prepared.request());
    assert_eq!(
        proof_packet.branch_basis(),
        &prepared.artifact().branch_basis
    );
    assert_eq!(
        proof_packet.execution_digest(),
        prepared
            .bound_executable_plan()
            .authority_binding
            .executable_plan_digest
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
        .execution_ready_plan()
        .lowered_records
        .iter()
        .map(|record| {
            RelationalMergeAdmittedSurfaceRow::new(
                record.record.clone(),
                record.target_record.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        proof_packet.admitted_merge_surface(),
        expected_surface.as_slice()
    );
    assert_eq!(
        proof_packet.admission_posture(),
        crate::facade::merge::RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted
    );
    assert!(!proof_packet.planning_digest().is_empty());
    assert!(!proof_packet
        .foundational_request_lowering_digest()
        .is_empty());
    assert!(!proof_packet.admitted_merge_surface_digest().is_empty());
    assert!(!proof_packet.packet_digest().is_empty());
}

#[test]
fn published_merge_outcome_retains_proof_packet_as_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let outcome = execute_merge(execution, &runtime);
    let summary_packet = outcome.execution_summary.proof_packet().clone();
    let live_authority = published_merge_authority(&runtime, outcome.commit.commit.commit_id);

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
    assert!(outcome
        .execution_summary
        .retains_consistent_proof_packet_authority());
    assert!(live_authority.retains_consistent_proof_packet_authority());
}

#[test]
fn proof_packet_deserialization_rejects_forged_retained_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    let proof_packet = runtime
        .merge()
        .retain_merge_proof_packet_from_prepared_execution(&prepared);

    let encoded = rmp_serde::to_vec_named(&proof_packet).expect("encode packet");
    let decoded: crate::facade::merge::RelationalMergeProofPacket =
        rmp_serde::from_slice(&encoded).expect("decode packet");
    assert_eq!(decoded, proof_packet);

    let forged_digest = rmp_serde::to_vec_named(&proof_packet_payload(
        &proof_packet,
        "forged-packet-digest",
        proof_packet.admitted_merge_surface_digest(),
    ))
    .expect("encode forged digest payload");
    let forged_digest_result: Result<crate::facade::merge::RelationalMergeProofPacket, _> =
        rmp_serde::from_slice(&forged_digest);
    assert!(forged_digest_result.is_err());

    let forged_surface = rmp_serde::to_vec_named(&proof_packet_payload(
        &proof_packet,
        proof_packet.packet_digest(),
        "forged-surface-digest",
    ))
    .expect("encode forged surface payload");
    let forged_surface_result: Result<crate::facade::merge::RelationalMergeProofPacket, _> =
        rmp_serde::from_slice(&forged_surface);
    assert!(forged_surface_result.is_err());

    let mut forged_rows = proof_packet.admitted_merge_surface().to_vec();
    forged_rows[0] = RelationalMergeAdmittedSurfaceRow::new(
        crate::transactions::data::RecordRef::Entity(crate::identity::data::EntityId::new(
            crate::identity::data::PartitionId::main(),
            999,
            1,
        )),
        None,
    );
    let forged_surface_payload = rmp_serde::to_vec_named(&proof_packet_payload_with_surface(
        &proof_packet,
        proof_packet.packet_digest(),
        proof_packet.admitted_merge_surface_digest(),
        &forged_rows,
    ))
    .expect("encode forged surface payload");
    let forged_surface_payload_result: Result<crate::facade::merge::RelationalMergeProofPacket, _> =
        rmp_serde::from_slice(&forged_surface_payload);
    assert!(forged_surface_payload_result.is_err());

    let malformed_execution_digest_packet =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            proof_packet.request().clone(),
            proof_packet.branch_basis().clone(),
            std::sync::Arc::from(proof_packet.admitted_merge_surface().to_vec()),
            proof_packet.correspondence_witness_digest().to_string(),
            proof_packet
                .schema_reconciliation_witness_digest()
                .to_string(),
            proof_packet.strategy_witness_digest().to_string(),
            proof_packet
                .foundational_request_lowering_digest()
                .to_string(),
            proof_packet.planning_digest().to_string(),
            "not-a-valid-sha256-digest".to_string(),
        );
    let malformed_execution_digest_payload =
        rmp_serde::to_vec_named(&malformed_execution_digest_packet)
            .expect("encode malformed execution digest packet");
    let malformed_execution_digest_result: Result<
        crate::facade::merge::RelationalMergeProofPacket,
        _,
    > = rmp_serde::from_slice(&malformed_execution_digest_payload);
    assert!(malformed_execution_digest_result.is_err());

    let malformed_planning_digest_packet =
        crate::facade::merge::RelationalMergeProofPacket::retained_execution_admitted(
            proof_packet.request().clone(),
            proof_packet.branch_basis().clone(),
            std::sync::Arc::from(proof_packet.admitted_merge_surface().to_vec()),
            proof_packet.correspondence_witness_digest().to_string(),
            proof_packet
                .schema_reconciliation_witness_digest()
                .to_string(),
            proof_packet.strategy_witness_digest().to_string(),
            proof_packet
                .foundational_request_lowering_digest()
                .to_string(),
            "still-not-a-valid-sha256-digest".to_string(),
            proof_packet.execution_digest().to_string(),
        );
    let malformed_planning_digest_payload =
        rmp_serde::to_vec_named(&malformed_planning_digest_packet)
            .expect("encode malformed planning digest packet");
    let malformed_planning_digest_result: Result<
        crate::facade::merge::RelationalMergeProofPacket,
        _,
    > = rmp_serde::from_slice(&malformed_planning_digest_payload);
    assert!(malformed_planning_digest_result.is_err());
}

#[test]
fn merge_proof_packet_survives_publication_and_recovery_without_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let outcome = execute_merge(execution, &runtime);
    let live_packet = outcome.execution_summary.proof_packet.clone();
    let live_authority = published_merge_authority(&runtime, outcome.commit.commit.commit_id);
    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let recovered_authority =
        published_merge_authority(&recovered, outcome.commit.commit.commit_id);

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

fn merge_ready_runtime(execution: &worth_execution::ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    runtime
}

fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}

fn execute_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergeExecutionOutcome {
    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, merge_request())
        .expect("prepared merge");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge")
}

fn published_merge_authority(
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(commit_id)
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}

#[derive(serde::Serialize)]
struct ProofPacketSerdePayload<'a> {
    request: &'a crate::facade::merge::NormalizedRelationalMergeRequest,
    branch_basis: &'a crate::history::data::RelationalMergeBranchBasis,
    admitted_merge_surface: &'a [RelationalMergeAdmittedSurfaceRow],
    correspondence_witness_digest: &'a str,
    schema_reconciliation_witness_digest: &'a str,
    strategy_witness_digest: &'a str,
    foundational_request_lowering_digest: &'a str,
    admitted_merge_surface_digest: &'a str,
    planning_digest: &'a str,
    execution_digest: &'a str,
    admission_posture: crate::facade::merge::RelationalMergeProofPacketAdmissionPosture,
    packet_digest: &'a str,
}

fn proof_packet_payload<'a>(
    packet: &'a crate::facade::merge::RelationalMergeProofPacket,
    packet_digest: &'a str,
    admitted_merge_surface_digest: &'a str,
) -> ProofPacketSerdePayload<'a> {
    proof_packet_payload_with_surface(
        packet,
        packet_digest,
        admitted_merge_surface_digest,
        packet.admitted_merge_surface(),
    )
}

fn proof_packet_payload_with_surface<'a>(
    packet: &'a crate::facade::merge::RelationalMergeProofPacket,
    packet_digest: &'a str,
    admitted_merge_surface_digest: &'a str,
    admitted_merge_surface: &'a [RelationalMergeAdmittedSurfaceRow],
) -> ProofPacketSerdePayload<'a> {
    ProofPacketSerdePayload {
        request: packet.request(),
        branch_basis: packet.branch_basis(),
        admitted_merge_surface,
        correspondence_witness_digest: packet.correspondence_witness_digest(),
        schema_reconciliation_witness_digest: packet.schema_reconciliation_witness_digest(),
        strategy_witness_digest: packet.strategy_witness_digest(),
        foundational_request_lowering_digest: packet.foundational_request_lowering_digest(),
        admitted_merge_surface_digest,
        planning_digest: packet.planning_digest(),
        execution_digest: packet.execution_digest(),
        admission_posture: packet.admission_posture(),
        packet_digest,
    }
}

#[path = "milestone_7e_phase_e/replay_drift.rs"]
mod replay_drift;
