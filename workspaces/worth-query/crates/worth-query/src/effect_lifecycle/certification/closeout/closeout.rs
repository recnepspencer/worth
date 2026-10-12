use crate::effect_lifecycle::{
    effect_lifecycle_public_surface_inventory, effect_lifecycle_support_matrix,
    EffectLifecycleCounters,
};
use crate::identity::hash_parts;

use super::closeout_artifacts::{
    EffectExecutionCertificationBundle, EffectExecutionCertificationLane,
    EffectExecutionCertificationOutputDigest, EffectExecutionCertificationRow,
};
use super::closeout_audits::build_closeout_audits;
use super::closeout_dx::build_closeout_dx;
use super::closeout_meta::build_closeout_meta_rows;
use super::closeout_oracles::build_closeout_oracles;
use super::closeout_postures::build_closeout_posture_rows;
use super::closeout_receipts::{
    batch_receipt_surface, mutation_receipt_surface, writeback_receipt_surface,
};
use super::closeout_slopes::build_closeout_slopes;
use super::{
    certify_effect_lifecycle_phase4, certify_effect_lifecycle_seeded, EffectLifecyclePhase4LaneKind,
};

pub fn certify_effect_execution_pipeline(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> EffectExecutionCertificationBundle {
    let seeded = certify_effect_lifecycle_seeded(execution, 17, 12);
    let phase4 = certify_effect_lifecycle_phase4(execution);
    let mutation = mutation_receipt_surface(execution);
    let writeback = writeback_receipt_surface(execution);
    let batch = batch_receipt_surface(execution);
    let support_matrix = effect_lifecycle_support_matrix();
    let public_surface = effect_lifecycle_public_surface_inventory();
    let dx = build_closeout_dx(execution, &public_surface);
    let oracles = build_closeout_oracles(execution);
    let audits = build_closeout_audits(
        &mutation,
        &writeback,
        &batch,
        seeded.certification_bundle_digest(),
        phase4.phase4_bundle_digest(),
    );
    let postures = build_closeout_posture_rows(&seeded, &phase4, &audits);
    let posture_failure_digest = postures.failure_digest();
    let slopes = build_closeout_slopes(
        &mutation,
        support_matrix.matrix_for_reporting(),
        dx.row.counter_snapshot_digest(),
    );
    let meta = build_closeout_meta_rows(&audits, &slopes, &mutation, &writeback, &batch, &dx);
    let closeout_counters = unique_closeout_counters(&mutation, &writeback, &batch, &dx);

    let rows = vec![
        mutation.as_row(EffectExecutionCertificationLane::MutationReceiptSurface),
        writeback.as_row(EffectExecutionCertificationLane::WritebackReceiptSurface),
        batch.as_row(EffectExecutionCertificationLane::BatchReceiptSurface),
        postures.advisory,
        postures.deferred,
        postures.denied,
        postures.mismatch,
        meta.proof_shape,
        meta.performance,
        dx.row,
        compile_fail_row(audits.compile_fail_boundary_digest().to_string()),
        seeded_replay_row(seeded.seed_replay_digest()),
        hostile_execution_row(&phase4),
    ];

    let outputs = vec![
        output("query_digest", mutation.query_digest.clone()),
        output("raw_effect_intent_digest", mutation.raw_digest.clone()),
        output(
            "normalized_effect_intent_digest",
            mutation.normalized_digest.clone(),
        ),
        output("effect_family_digest", mutation.family_digest.clone()),
        output("effect_authority_digest", mutation.authority_digest.clone()),
        output("effect_basis_digest", mutation.basis_digest.clone()),
        output("effect_scope_digest", mutation.scope_digest.clone()),
        output("effect_policy_digest", mutation.policy_digest.clone()),
        output("effect_strategy_digest", mutation.strategy_digest.clone()),
        output(
            "effect_eligibility_digest",
            mutation.eligibility_digest.clone(),
        ),
        output(
            "authority_scoped_effect_plan_digest",
            mutation.plan_digest.clone(),
        ),
        output(
            "lowered_effect_execution_plan_digest",
            mutation.lowered_digest.clone(),
        ),
        output(
            "effect_execution_receipt_digest",
            mutation.receipt_digest.clone(),
        ),
        output("effect_envelope_digest", mutation.envelope_digest.clone()),
        output(
            "relational_effect_authority_digest",
            mutation.authority_artifact_digest.clone(),
        ),
        output(
            "bridge_effect_authority_digest",
            writeback.authority_artifact_digest.clone(),
        ),
        output(
            "effect_decision_trace_digest",
            mutation.decision_trace_digest.clone(),
        ),
        output(
            "effect_structural_delta_digest",
            mutation.structural_delta_digest.clone(),
        ),
        output(
            "effect_integrity_marker_digest",
            mutation.integrity_digest.clone(),
        ),
        output("effect_target_dx_digest", dx.target_dx_digest),
        output(
            "effect_golden_transcript_digest",
            dx.golden_transcript_digest,
        ),
        output(
            "effect_support_matrix_digest",
            support_matrix.matrix_for_reporting().to_string(),
        ),
        output(
            "effect_proof_shape_digest",
            audits.proof_shape_digest().to_string(),
        ),
        output(
            "effect_phase_progression_digest",
            audits.phase_progression_digest().to_string(),
        ),
        output(
            "effect_replay_parity_digest",
            seeded.seed_replay_digest().to_string(),
        ),
        output(
            "relational_oracle_digest",
            oracles.relational_oracle_digest().to_string(),
        ),
        output(
            "bridge_oracle_digest",
            oracles.bridge_oracle_digest().to_string(),
        ),
        output(
            "seeded_sequence_digest",
            seeded.seeded_sequence_digest().to_string(),
        ),
        output(
            "seed_replay_digest",
            seeded.seed_replay_digest().to_string(),
        ),
        output(
            "compile_fail_boundary_digest",
            audits.compile_fail_boundary_digest().to_string(),
        ),
        output("failure_digest", posture_failure_digest),
        output("counter_snapshot", counters_digest(&rows)),
        output(
            "executor_rediscovery_count",
            closeout_counters
                .effect_executor_rediscovery_count()
                .to_string(),
        ),
        output(
            "batch_lowering_count",
            closeout_counters.batch_lowering_count().to_string(),
        ),
        output(
            "batch_basis_reuse_count",
            closeout_counters.batch_basis_reuse_count().to_string(),
        ),
        output(
            "authority_reopen_count",
            closeout_counters.authority_reopen_count().to_string(),
        ),
        output(
            "effect_normalization_slope_digest",
            slopes.normalization().to_string(),
        ),
        output(
            "effect_eligibility_slope_digest",
            slopes.eligibility().to_string(),
        ),
        output(
            "effect_lowering_slope_digest",
            slopes.lowering().to_string(),
        ),
        output(
            "effect_execution_slope_digest",
            slopes.execution().to_string(),
        ),
        output(
            "effect_receipt_materialization_slope_digest",
            slopes.receipt_materialization().to_string(),
        ),
        output(
            "effect_envelope_materialization_slope_digest",
            slopes.envelope_materialization().to_string(),
        ),
        output(
            "effect_support_lookup_slope_digest",
            slopes.support_lookup().to_string(),
        ),
    ];

    EffectExecutionCertificationBundle::new(
        rows,
        outputs,
        seeded.certification_bundle_digest().to_string(),
        phase4.phase4_bundle_digest().to_string(),
    )
}

fn compile_fail_row(compile_fail_digest: String) -> EffectExecutionCertificationRow {
    EffectExecutionCertificationRow::new(
        EffectExecutionCertificationLane::CompileFailBoundary,
        compile_fail_digest,
        "effect lifecycle compile-fail boundary suite".to_string(),
        &EffectLifecycleCounters::default(),
        None,
    )
}

fn seeded_replay_row(seed_replay_digest: &str) -> EffectExecutionCertificationRow {
    EffectExecutionCertificationRow::new(
        EffectExecutionCertificationLane::SeededReplayParity,
        seed_replay_digest.to_string(),
        "seeded replay bundle remains deterministic".to_string(),
        &EffectLifecycleCounters::default(),
        None,
    )
}

fn hostile_execution_row(
    phase4: &super::EffectLifecyclePhase4CertificationBundle,
) -> EffectExecutionCertificationRow {
    let hostile_rows = phase4
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.lane_kind(),
                EffectLifecyclePhase4LaneKind::BatchLaneDenial
                    | EffectLifecyclePhase4LaneKind::PreviewRebind
                    | EffectLifecyclePhase4LaneKind::HostOverrideDenial
                    | EffectLifecyclePhase4LaneKind::StaleAfterAdmission
                    | EffectLifecyclePhase4LaneKind::StaleAfterLowering
            )
        })
        .collect::<Vec<_>>();
    let evidence_digest = hash_parts(
        &hostile_rows
            .iter()
            .map(|row| row.row_digest().to_string())
            .collect::<Vec<_>>(),
    );
    let detail = hostile_rows
        .iter()
        .map(|row| row.lane_kind().as_str())
        .collect::<Vec<_>>()
        .join("|");
    let counters = hostile_rows
        .first()
        .map(|row| row.counters().clone())
        .unwrap_or_default();
    EffectExecutionCertificationRow::new(
        EffectExecutionCertificationLane::HostileExecutionSurface,
        evidence_digest,
        detail,
        &counters,
        None,
    )
}

fn output(name: &'static str, digest: String) -> EffectExecutionCertificationOutputDigest {
    EffectExecutionCertificationOutputDigest::certified(name, digest)
}

fn counters_digest(rows: &[EffectExecutionCertificationRow]) -> String {
    hash_parts(
        &rows
            .iter()
            .map(|row| row.counter_snapshot_digest().to_string())
            .collect::<Vec<_>>(),
    )
}

fn unique_closeout_counters(
    mutation: &super::closeout_receipts::ReceiptSurfaceEvidence,
    writeback: &super::closeout_receipts::ReceiptSurfaceEvidence,
    batch: &super::closeout_receipts::ReceiptSurfaceEvidence,
    dx: &super::closeout_dx::CloseoutDxEvidence,
) -> EffectLifecycleCounters {
    mutation
        .counters
        .combine(&writeback.counters)
        .combine(&batch.counters)
        .combine(&dx.support_lookup_counters)
}
