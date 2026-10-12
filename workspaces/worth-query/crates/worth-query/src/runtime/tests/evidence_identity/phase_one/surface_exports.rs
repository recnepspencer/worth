use super::*;

#[test]
fn runtime_evidence_identity_surface_is_exported_through_public_facades() {
    let identity = crate::WorthQueryEvidenceIdentityScheme::V1;
    let runtime_identity = crate::facade::runtime::WorthQueryEvidenceIdentityScheme::V1;

    assert_eq!(identity, runtime_identity);
    assert_eq!(
        crate::WorthQueryEvidenceScope::RuntimeStateSnapshot,
        crate::facade::runtime::WorthQueryEvidenceScope::RuntimeStateSnapshot
    );
    let manual = crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::RuntimeStateSnapshot,
    )
    .field_shape(crate::WorthQueryEvidenceTag::new("kind"), "ready")
    .seal();
    assert_canonical_evidence_identity_token(manual.as_str());
}

#[test]
fn support_matrix_and_state_snapshot_emit_canonical_evidence_tokens() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_backed_runtime_with_support(request_execution, intent_support_profile());
    let workspace =
        WorthQueryWorkspace::new("evidence-identity-support", runtime).expect("workspace builds");
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
        assert_canonical_evidence_identity_token(
            row.row_digest().terminal_projection_for_reporting(),
        );
        assert_eq!(
            row.row_digest().as_str(),
            compose_public_support_matrix_row_identity(row).as_str()
        );
    }
    assert_eq!(
        matrix.matrix_digest().as_str(),
        compose_public_support_matrix_identity(&matrix).as_str()
    );
    assert_canonical_evidence_identity_token(
        snapshot.state_digest().terminal_projection_for_reporting(),
    );
    assert_eq!(
        snapshot.state_digest().as_str(),
        compose_state_snapshot_identity(&snapshot).as_str()
    );
}

#[test]
fn state_snapshot_with_optional_postures_recomposes_exactly() {
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
