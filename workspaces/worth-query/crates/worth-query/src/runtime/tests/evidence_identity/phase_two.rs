use super::super::support::*;
use super::proof_support::*;
use crate::runtime::evidence_identities::{
    runtime_state_snapshot_basis_label_identity, runtime_state_snapshot_result_shape_label_identity,
};

#[test]
fn support_matrix_and_state_snapshot_remain_in_phase_two_canonical_migration_coverage() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_backed_runtime_with_support(request_execution, intent_support_profile());
    let workspace =
        WorthQueryWorkspace::new("phase-two-evidence-identity", runtime).expect("workspace builds");
    let matrix = workspace.public_support_matrix();
    let snapshot = WorthQueryRuntimeStateSnapshot::ready(
        runtime_state_snapshot_basis_label_identity(
            &crate::runtime::evidence_identities::runtime_state_snapshot_test_subject_identity(
                "basis|digest",
            ),
        ),
        runtime_state_snapshot_result_shape_label_identity(
            &crate::runtime::evidence_identities::runtime_state_snapshot_test_subject_identity(
                "result:shape",
            ),
        ),
        WorthQueryAuthorityLane::PreviewTruth,
        "state explanation with | and : punctuation",
    );

    assert_canonical_evidence_identity_token(
        matrix.matrix_digest().terminal_projection_for_reporting(),
    );
    for row in matrix.rows() {
        assert_eq!(
            row.row_digest().as_str(),
            compose_public_support_matrix_row_identity(row).as_str()
        );
    }
    assert_eq!(
        matrix.matrix_digest().as_str(),
        compose_public_support_matrix_identity(&matrix).as_str()
    );
    assert_eq!(
        snapshot.state_digest().as_str(),
        compose_state_snapshot_identity(&snapshot).as_str()
    );
}

#[test]
fn state_snapshot_with_optional_postures_remains_in_phase_two_canonical_migration_coverage() {
    let ordinary_runtime_posture = crate::ordinary_outcome::WorthQueryOrdinaryRuntimePosture::new(
        crate::ordinary_outcome::WorthQueryOrdinaryRuntimePostureKind::Revalidating,
        crate::ordinary_outcome::WorthQueryOrdinaryRuntimeCausePostureKind::MixedCause,
        Some(crate::ordinary_outcome::WorthQueryOrdinaryRuntimeAsyncPostureKind::Revalidating),
        crate::ordinary_outcome::WorthQueryOrdinaryRuntimeBasisPostureKind::GenerationDrift,
        Some(
            crate::ordinary_outcome::WorthQueryOrdinaryRuntimeRemaskPostureKind::SchemaContextDrift,
        ),
        "support|evidence|digest",
    );
    let async_result_state = WorthQueryRuntimeAsyncResultState::new(
        WorthQueryRuntimeAsyncResultStateKind::Revalidating,
        &crate::runtime::async_result_state::runtime_async_causality_from_label("causality|digest"),
        &crate::runtime::async_result_state::runtime_async_checkpoint_label_identity(
            "basis|digest",
        ),
        &crate::runtime::async_result_state::runtime_async_checkpoint_label_identity(
            "generation|digest",
        ),
    );
    let remask_projection = WorthQueryRuntimeRemaskProjection::remasked(
        WorthQueryRuntimeRemaskReasonKind::SchemaContextDrift,
        "policy|digest",
        "tenant-truth|digest",
        "tenant-schema|digest",
        "relationship-proof|digest",
        "schema-context|digest",
    );
    let remask_posture = WorthQueryRuntimeRemaskPosture::from_activation_projection(
        &remask_projection,
        &runtime_state_snapshot_basis_label_identity(
            &crate::runtime::evidence_identities::runtime_state_snapshot_test_subject_identity(
                "support|evidence|digest",
            ),
        ),
        &runtime_state_snapshot_basis_label_identity(
            &crate::runtime::evidence_identities::runtime_state_snapshot_test_subject_identity(
                "basis|digest",
            ),
        ),
    );
    let snapshot = WorthQueryRuntimeStateSnapshot::ready(
        runtime_state_snapshot_basis_label_identity(
            &crate::runtime::evidence_identities::runtime_state_snapshot_test_subject_identity(
                "basis|digest",
            ),
        ),
        runtime_state_snapshot_result_shape_label_identity(
            &crate::runtime::evidence_identities::runtime_state_snapshot_test_subject_identity(
                "result:shape",
            ),
        ),
        WorthQueryAuthorityLane::BridgeExternalState,
        "state explanation with optional posture pressure",
    )
    .with_ordinary_runtime_posture(ordinary_runtime_posture)
    .with_async_result_state(async_result_state)
    .with_remask_posture(remask_posture);

    assert_canonical_evidence_identity_token(
        snapshot.state_digest().terminal_projection_for_reporting(),
    );
    assert_eq!(
        snapshot.state_digest().as_str(),
        compose_state_snapshot_identity(&snapshot).as_str()
    );
}

#[test]
fn public_api_contract_transcript_and_support_report_emit_canonical_evidence_tokens() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_runtime(request_execution);
    let contract = runtime.public_api_contract();
    assert_canonical_evidence_identity_token(contract.contract_digest());
    assert_eq!(
        contract.contract_identity().as_str(),
        contract.contract_digest()
    );
    assert_eq!(
        contract.contract_digest(),
        compose_public_api_contract_identity(&contract).as_str()
    );
    for family in contract.families() {
        assert_canonical_evidence_identity_token(family.contract_digest());
        assert_eq!(
            family.contract_identity().as_str(),
            family.contract_digest()
        );
        assert_eq!(
            family.contract_digest(),
            compose_public_api_family_contract_identity(family).as_str()
        );
    }

    let transcript =
        crate::harness::RuntimeApiStabilizationAdapter::composed_runtime_hostile_transcript_evidence(request_execution);
    assert_canonical_evidence_identity_token(transcript.transcript_digest());
    assert_eq!(
        transcript.transcript_identity().as_str(),
        transcript.transcript_digest()
    );
    assert_eq!(
        transcript.transcript_digest(),
        compose_runtime_public_api_transcript_identity(&transcript).as_str()
    );
    assert!(
        transcript.support_gated_neighbor_denial_digests().len() >= 2,
        "ordinary runtime-backed transcript should carry multiple support-gated neighbor denials"
    );
    assert!(
        transcript.delivery_residue_count() >= 1,
        "ordinary runtime-backed transcript should prove delivery residue through the public transcript surface"
    );
    assert!(
        transcript.meaningful_assertion_count() >= 16,
        "ordinary runtime-backed transcript should remain a certification-grade proof artifact"
    );

    let report =
        crate::application::WorthQueryApplicationFacade::runtime_backed_default().support_report();
    assert_canonical_evidence_identity_token(report.report_digest());
    assert_eq!(report.report_identity().as_str(), report.report_digest());
    for posture in report.section_postures() {
        assert_eq!(
            posture.posture_identity().as_str(),
            posture.posture_digest()
        );
    }
    assert_eq!(
        report.report_digest(),
        compose_support_report_identity(&report).as_str()
    );

    let query_disabled_report = crate::application::WorthQueryApplicationFacade::new(
        crate::application::WorthQueryConfig::runtime_backed_default()
            .with_query(crate::application::WorthQueryQueryConfig::disabled())
            .with_signal(crate::application::WorthQuerySignalConfig::disabled())
            .with_runtime_bridge(crate::application::WorthQueryRuntimeBridgeConfig::disabled())
            .with_relational(crate::application::WorthQueryRelationalConfig::disabled()),
    )
    .expect("query-disabled facade config should remain valid")
    .support_report();
    assert_canonical_evidence_identity_token(query_disabled_report.report_digest());
    assert_eq!(
        query_disabled_report.report_digest(),
        compose_support_report_identity(&query_disabled_report).as_str()
    );
    assert_ne!(
        report.report_digest(),
        query_disabled_report.report_digest()
    );
}
