use super::super::super::support::*;

pub(in super::super) fn compose_public_api_family_contract_identity(
    contract: &WorthQueryRuntimePublicApiFamilyContract,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::RuntimePublicApiFamilyContract,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("family"),
        contract.family().as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("status"),
        contract.status().as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("teaching_posture"),
        contract.teaching_posture().as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("owner_closure"),
        contract.owner_closure(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("extension_rule"),
        contract.extension_rule(),
    )
    .field_bool(
        crate::WorthQueryEvidenceTag::new("parallel_api_forbidden"),
        contract.parallel_api_forbidden(),
    )
    .field_bool(
        crate::WorthQueryEvidenceTag::new("admission_fail_closed"),
        contract.admission_fail_closed(),
    )
    .field_value_sequence(
        crate::WorthQueryEvidenceTag::new("authority_lanes"),
        contract.authority_lanes().iter().map(|lane| lane.as_str()),
    )
    .field_value_sequence(
        crate::WorthQueryEvidenceTag::new("evidence"),
        contract.evidence().iter().map(String::as_str),
    )
    .optional_value(
        crate::WorthQueryEvidenceTag::new("reason"),
        contract.reason(),
    )
    .seal()
    .expect("fixture digest must fit")
}

pub(in super::super) fn compose_public_api_contract_identity(
    contract: &WorthQueryRuntimePublicApiContract,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::RuntimePublicApiContract,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("backend_posture"),
        contract.backend_posture().as_str(),
    )
    .field_value_sequence(
        crate::WorthQueryEvidenceTag::new("family_contract_digest"),
        contract
            .families()
            .iter()
            .map(|family| family.contract_digest()),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("stable_family_count"),
        contract.stable_family_count(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("deferred_family_count"),
        contract.deferred_family_count(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("unsupported_family_count"),
        contract.unsupported_family_count(),
    )
    .seal()
    .expect("fixture digest must fit")
}

pub(in super::super) fn compose_public_support_matrix_row_identity(
    row: &WorthQueryRuntimePublicSupportMatrixRow,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::RuntimePublicSupportMatrixRow,
    )
    .field_shape(crate::WorthQueryEvidenceTag::new("surface"), row.surface())
    .field_shape(
        crate::WorthQueryEvidenceTag::new("facade_family"),
        row.facade_family()
            .map(WorthQueryRuntimeFacadeFamily::as_str)
            .unwrap_or("matrix-only"),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("status"),
        row.status().as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("teaching_posture"),
        row.teaching_posture().as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("owner_milestone"),
        row.owner_milestone(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("extension_rule"),
        row.extension_rule(),
    )
    .field_bool(
        crate::WorthQueryEvidenceTag::new("parallel_api_forbidden"),
        row.parallel_api_forbidden(),
    )
    .field_bool(
        crate::WorthQueryEvidenceTag::new("admission_fail_closed"),
        row.admission_fail_closed(),
    )
    .optional_value(
        crate::WorthQueryEvidenceTag::new("support_contract_digest"),
        row.support_contract_digest(),
    )
    .seal()
    .expect("fixture digest must fit")
}

pub(in super::super) fn compose_public_support_matrix_identity(
    matrix: &WorthQueryRuntimePublicSupportMatrix,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::RuntimePublicSupportMatrix,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("backend_posture"),
        matrix.backend_posture().as_str(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("stable_row_count"),
        matrix.stable_row_count(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("deferred_row_count"),
        matrix.deferred_row_count(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("unsupported_row_count"),
        matrix.unsupported_row_count(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("fail_closed_row_count"),
        matrix.fail_closed_row_count(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("parallel_api_forbidden_row_count"),
        matrix.parallel_api_forbidden_row_count(),
    )
    .field_value_sequence(
        crate::WorthQueryEvidenceTag::new("row_digest"),
        matrix.rows().iter().map(|row| row.row_digest().as_str()),
    )
    .seal()
    .expect("fixture digest must fit")
}

pub(in super::super) fn compose_state_snapshot_identity(
    snapshot: &WorthQueryRuntimeStateSnapshot,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(crate::WorthQueryEvidenceScope::RuntimeStateSnapshot)
        .field_shape(
            crate::WorthQueryEvidenceTag::new("kind"),
            snapshot.kind().as_str(),
        )
        .field_evidence_identity(
            crate::WorthQueryEvidenceTag::new("basis_digest"),
            snapshot.basis_identity(),
        )
        .field_evidence_identity(
            crate::WorthQueryEvidenceTag::new("result_shape_digest"),
            snapshot.result_shape_identity(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("authority_lane"),
            snapshot.authority_lane().as_str(),
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("explanation"),
            snapshot.explanation(),
        )
        .optional_evidence_identity(
            crate::WorthQueryEvidenceTag::new("ordinary_runtime_posture"),
            snapshot
                .ordinary_runtime_posture()
                .map(crate::ordinary_outcome::WorthQueryOrdinaryRuntimePosture::evidence_identity),
        )
        .optional_evidence_identity(
            crate::WorthQueryEvidenceTag::new("async_result_state"),
            snapshot
                .async_result_state()
                .map(WorthQueryRuntimeAsyncResultState::result_state_identity),
        )
        .optional_evidence_identity(
            crate::WorthQueryEvidenceTag::new("remask_posture"),
            snapshot
                .remask_posture()
                .map(WorthQueryRuntimeRemaskPosture::remask_identity),
        )
        .seal()
        .expect("fixture digest must fit")
}

pub(in super::super) fn compose_runtime_public_api_transcript_identity(
    transcript: &WorthQueryRuntimePublicApiTranscriptEvidence,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::RuntimePublicApiTranscriptEvidence,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("transcript_family"),
        transcript.transcript_family(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("support_contract_digest"),
        transcript.support_contract_digest(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("state_digest"),
        transcript.state_digest(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("live_surface_digest"),
        transcript.live_surface_digest(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("computed_surface_digest"),
        transcript.computed_surface_digest(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("effect_surface_digest"),
        transcript.effect_surface_digest(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("intent_receipt_digest"),
        transcript.intent_receipt_digest(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("inspection_digest"),
        transcript.inspection_digest(),
    )
    .field_value_sequence(
        crate::WorthQueryEvidenceTag::new("support_gated_neighbor_denial_digest"),
        transcript
            .support_gated_neighbor_denial_digests()
            .iter()
            .map(String::as_str),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("delivery_residue_count"),
        transcript.delivery_residue_count(),
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("authority_lane_digest"),
        transcript.authority_lane_digest(),
    )
    .field_usize(
        crate::WorthQueryEvidenceTag::new("meaningful_assertion_count"),
        transcript.meaningful_assertion_count(),
    )
    .seal()
    .expect("fixture digest must fit")
}
