use super::*;

mod proof_steps;

use proof_steps::{
    establish_feedback_origin_proof, execute_replayed_feedback_authority,
    publish_interleaved_feedback_proof, rebuild_feedback_replay_proof,
    reject_changed_effect_feedback, verify_replayed_feedback_context,
};

pub(super) fn execute_feedback_loop_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let origin = establish_feedback_origin_proof(runtime_bridge, fixture, resource_request)?;
    let publication = publish_interleaved_feedback_proof(
        runtime,
        runtime_bridge,
        &origin.original_commit,
        &origin.feedback_context,
        resource_request,
    )?;
    let replay_context = verify_replayed_feedback_context(
        runtime_bridge,
        &origin.original_commit,
        &origin.initial_route_digest,
        &origin.effect,
        &publication.carried_feedback_context,
    )?;
    let changed_effect_denial = reject_changed_effect_feedback(
        runtime_bridge,
        &origin.contract,
        &origin.lowered_policy_bundle,
        &replay_context.replayed_causality,
        &origin.effect,
        &publication.carried_feedback_context,
    );
    let replayed_authority = execute_replayed_feedback_authority(
        runtime_bridge,
        &origin.contract,
        &origin.lowered_policy_bundle,
        &origin.effect,
        &publication.carried_feedback_context,
        &replay_context.replayed_feedback_provenance,
    )?;
    let restart = rebuild_feedback_replay_proof(
        runtime,
        runtime_bridge,
        fixture,
        &replay_context.replayed_causality,
        &publication.carried_feedback_context,
    )?;

    let changed_effect_feedback_matches_initial = runtime_bridge
        .derive_writeback_feedback_provenance(&changed_effect_denial.changed_effect)
        .digest()
        == origin.feedback_provenance.digest();

    Ok(WritebackHarnessExecution::FeedbackLoopCertification {
        feedback_loop_digest: feedback_loop_digest(
            &origin.initial_outcome,
            &replayed_authority.replayed_outcome,
            &replayed_authority.replayed_bundle,
        ),
        feedback_route_identity: publication.feedback_route_identity.clone(),
        feedback_origin_matrix: WritebackFeedbackLoopMatrix::from_feedback_evidence(
            WritebackFeedbackLoopMatrixEvidence {
                contract: &origin.contract,
                effect: &origin.effect,
                original_causality: &origin.original_causality,
                replayed_bundle: &replayed_authority.replayed_bundle,
                initial_outcome: &origin.initial_outcome,
                initial_idempotence: &origin.initial_idempotence,
                replayed_idempotence: &replayed_authority.replayed_idempotence,
                loop_prevention: &replayed_authority.loop_prevention,
                replayed_strategy_coherence: &replayed_authority.replayed_strategy_coherence,
                replayed_candidate: replayed_authority.replayed_candidate.as_ref(),
                feedback_authority_request: replayed_authority.feedback_authority_request.as_ref(),
                replayed_receipt: replayed_authority.replayed_receipt.as_ref(),
                changed_effect: &changed_effect_denial.changed_effect,
                changed_idempotence: &changed_effect_denial.changed_idempotence,
                changed_effect_error: &changed_effect_denial.changed_effect_error,
                changed_effect_feedback_matches_initial,
                ordinary_truth_commit_identity: &publication.ordinary_commit_identity,
                ordinary_route_identity: &publication.ordinary_route_identity,
                rebuilt_contract: &restart.rebuilt_contract,
                rebuilt_effect: &restart.rebuilt_effect,
                rebuilt_idempotence: &restart.rebuilt_idempotence,
                rebuilt_loop_prevention: &restart.rebuilt_loop_prevention,
                rebuilt_outcome: &restart.rebuilt_outcome,
                rebuilt_replay_bundle: &restart.rebuilt_replay_bundle,
                rebuilt_receipt: restart.rebuilt_receipt.as_ref(),
                feedback_provenance: &origin.feedback_provenance,
                carried_feedback_context: &publication.carried_feedback_context,
                feedback_commit_identity: &publication.feedback_commit_identity,
                feedback_route_identity: &publication.feedback_route_identity,
                authoritative_commit_count: runtime.writeback_authority.committed_causality_count(),
            },
        ),
        counter_snapshot: restart.counter_snapshot,
    })
}

fn feedback_loop_digest(
    initial_outcome: &crate::facade::BridgeWritebackAuthorityOutcome,
    replayed_outcome: &crate::facade::BridgeWritebackAuthorityOutcome,
    replayed_bundle: &crate::facade::BridgeWritebackReplayBundle,
) -> String {
    digest_string(
        "bridge-writeback-feedback-loop",
        &format!(
            "initial-outcome={}|replayed-outcome={}|replayed-bundle={}",
            initial_outcome.digest(),
            replayed_outcome.digest(),
            replayed_bundle.digest(),
        ),
    )
    .to_string()
}
