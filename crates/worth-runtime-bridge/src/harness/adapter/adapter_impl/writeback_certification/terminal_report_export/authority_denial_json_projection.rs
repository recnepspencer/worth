use serde_json::json;

use super::super::super::terminal_report_export::writeback_counter_snapshot_json;
use super::super::authority_denial::{
    AuthorityDenialBoundaryFailure, AuthorityDenialBoundaryMatrix,
    AuthorityDenialLoopPreventionEvidence, AuthorityDenialZeroResidueProof,
    WritebackAuthorityDenialMatrix,
};

pub(in crate::harness::adapter::adapter_impl) fn authority_denial_matrix_json(
    matrix: &WritebackAuthorityDenialMatrix,
) -> serde_json::Value {
    json!({
        "writeback_digest": serde_json::Value::Null,
        "effect_intent_digest": serde_json::Value::Null,
        "causality_digest": serde_json::Value::Null,
        "replay_digest": serde_json::Value::Null,
        "mutation_plan_digest": serde_json::Value::Null,
        "idempotence_report": serde_json::Value::Null,
        "loop_prevention_report": loop_prevention_report_json(matrix),
        "authority_boundary_matrix": authority_boundary_json(matrix.authority_boundary()),
        "failure_kind": format!("{:?}", matrix.validation_failure_kind()),
        "detail": matrix.validation_detail(),
        "typed_boundary": "preview-writeback-validation",
        "validation_error_kind": format!("{:?}", matrix.validation_failure_kind()),
    })
}

pub(in crate::harness::adapter::adapter_impl) fn authority_denial_certification_evidence_json(
    matrix: &WritebackAuthorityDenialMatrix,
    zero_residue_proof: AuthorityDenialZeroResidueProof,
    failure_digest: &str,
    counter_snapshot: &crate::facade::BridgeWritebackCounters,
    counter_digest: &str,
) -> serde_json::Value {
    json!({
        "certification_shape": "authority-denial-certification",
        "writeback_digest": serde_json::Value::Null,
        "effect_intent_digest": serde_json::Value::Null,
        "causality_digest": serde_json::Value::Null,
        "mutation_plan_digest": serde_json::Value::Null,
        "idempotence_report": serde_json::Value::Null,
        "loop_prevention_report": loop_prevention_report_json(matrix),
        "truth_integrity_report": authority_denial_zero_residue_proof_json(zero_residue_proof),
        "authority_boundary_matrix": authority_boundary_json(matrix.authority_boundary()),
        "failure_digest": failure_digest,
        "replay_digest": serde_json::Value::Null,
        "counter_snapshot": writeback_counter_snapshot_json(counter_snapshot),
        "counter_digest": counter_digest,
    })
}

pub(in crate::harness::adapter::adapter_impl) fn authority_denial_zero_residue_proof_json(
    proof: AuthorityDenialZeroResidueProof,
) -> serde_json::Value {
    json!({
        "authoritative_commit_count": proof.authoritative_commit_count(),
        "authoritative_artifact_count": proof.authoritative_artifact_count(),
        "retained_writeback_bundle_count": proof.retained_writeback_bundle_count(),
        "loop_side_effect_count": proof.loop_side_effect_count(),
    })
}

fn loop_prevention_report_json(matrix: &WritebackAuthorityDenialMatrix) -> serde_json::Value {
    json!({
        "unsafe_feedback_partial": loop_prevention_json(matrix.unsafe_feedback_partial()),
        "unsafe_feedback_contradictory": loop_prevention_json(matrix.unsafe_feedback_contradictory()),
    })
}

fn authority_boundary_json(matrix: &AuthorityDenialBoundaryMatrix) -> serde_json::Value {
    json!({
        "preview_validation_failure": boundary_failure_json(matrix.preview_validation_failure()),
        "unbound_authority_failure": boundary_failure_json(matrix.unbound_authority_failure()),
        "unsafe_feedback_failure": boundary_failure_json(matrix.unsafe_feedback_failure()),
        "contradictory_feedback_failure": boundary_failure_json(matrix.contradictory_feedback_failure()),
    })
}

fn loop_prevention_json(
    loop_prevention: &AuthorityDenialLoopPreventionEvidence,
) -> serde_json::Value {
    json!({
        "digest": loop_prevention.digest(),
        "disposition": format!("{:?}", loop_prevention.disposition()),
        "current_feedback_provenance_digest": loop_prevention.current_feedback_provenance_digest(),
        "current_causality_digest": loop_prevention.current_causality_digest(),
        "incoming_feedback_provenance_digest": loop_prevention.incoming_feedback_provenance_digest(),
        "incoming_feedback_causality_digest": loop_prevention.incoming_feedback_causality_digest(),
    })
}

fn boundary_failure_json(failure: &AuthorityDenialBoundaryFailure) -> serde_json::Value {
    json!({
        "contract_digest": failure.contract_digest(),
        "strategy_basis_digest": failure.strategy_basis_digest(),
        "strategy_coherence_digest": failure.strategy_coherence_digest(),
        "strategy_coherence_disposition": failure
            .strategy_coherence_disposition()
            .map(|disposition| format!("{disposition:?}")),
        "denial_class": failure.denial_class().as_str(),
        "failure_kind": format!("{:?}", failure.failure_kind()),
        "failure_digest": failure.failure_digest(),
        "causality_digest": failure.causality_digest(),
        "writeback_effect_artifact_digest": failure.writeback_effect_artifact_digest(),
        "effect_intent_digest": failure.effect_intent_digest(),
        "idempotence_digest": failure.idempotence_digest(),
        "feedback_provenance_digest": failure.feedback_provenance_digest(),
        "incoming_feedback_causality_digest": failure.incoming_feedback_causality_digest(),
    })
}
