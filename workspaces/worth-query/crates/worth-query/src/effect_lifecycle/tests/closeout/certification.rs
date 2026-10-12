use crate::effect_lifecycle::{
    certify_effect_execution_pipeline, EffectExecutionCertificationLane,
};

use super::super::certification::{
    certify_effect_lifecycle_phase4, certify_effect_lifecycle_seeded,
    EffectLifecyclePhase4LaneKind, EffectLifecyclePhase4LaneOutcome,
    EffectLifecycleSeededOutcomeClass,
};

#[test]
fn seeded_certification_replays_the_same_seed_identically() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = certify_effect_lifecycle_seeded(request_execution, 17, 12);
    let right = certify_effect_lifecycle_seeded(request_execution, 17, 12);

    assert_eq!(
        left.seeded_sequence_digest(),
        right.seeded_sequence_digest()
    );
    assert_eq!(left.seed_replay_digest(), right.seed_replay_digest());
    assert_eq!(
        left.certification_bundle_digest(),
        right.certification_bundle_digest()
    );
    assert!(left.replay_is_deterministic());
    assert!(right.replay_is_deterministic());
}

#[test]
fn seeded_certification_changes_when_the_seed_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = certify_effect_lifecycle_seeded(request_execution, 17, 12);
    let right = certify_effect_lifecycle_seeded(request_execution, 18, 12);

    assert_ne!(
        left.seeded_sequence_digest(),
        right.seeded_sequence_digest()
    );
    assert_ne!(
        left.certification_bundle_digest(),
        right.certification_bundle_digest()
    );
}

#[test]
fn seeded_certification_covers_distinct_postures_and_batch_widths() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_effect_lifecycle_seeded(request_execution, 17, 12);
    let rows = bundle.rows();

    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::ScalarExecuted));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::BatchExecuted));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::Advisory));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::RebindRequired));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::Deferred));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::Denied));
    assert!(rows.iter().any(|row| row.batch_width() > 1));
    assert!(rows.iter().all(|row| !row.row_digest().is_empty()));
    assert!(rows
        .iter()
        .all(|row| !row.counter_snapshot_digest().is_empty()));
    assert!(rows
        .iter()
        .any(|row| row.counters().effect_executor_rediscovery_count() == 0));
    assert!(rows
        .iter()
        .any(|row| row.counters().batch_lowering_count() == 1));
}

#[test]
fn seeded_certification_never_certifies_less_than_full_template_coverage() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_effect_lifecycle_seeded(request_execution, 17, 1);
    let rows = bundle.rows();

    assert!(rows.len() >= 9);
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::ScalarExecuted));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::BatchExecuted));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::Advisory));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::RebindRequired));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::Deferred));
    assert!(rows
        .iter()
        .any(|row| row.outcome_class() == EffectLifecycleSeededOutcomeClass::Denied));
}

#[test]
fn phase4_certification_covers_named_execution_and_boundary_lanes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_effect_lifecycle_phase4(request_execution);
    let rows = bundle.rows();

    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::BranchMutationExecution
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Executed));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::RelationalMergeExecution
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Executed));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::BridgeWritebackExecution
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Executed));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::BatchExecution
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Executed));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::BatchLaneDenial
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Denied));
    assert!(rows.iter().any(
        |row| row.lane_kind() == EffectLifecyclePhase4LaneKind::PreviewRebind
            && row.outcome() == EffectLifecyclePhase4LaneOutcome::RebindRequired
    ));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::DeferredReplay
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Deferred));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::HostOverrideDenial
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Denied));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::StaleAfterAdmission
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Denied));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::StaleAfterLowering
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Denied));
    assert!(rows.iter().any(|row| row.lane_kind()
        == EffectLifecyclePhase4LaneKind::RelationalOracle
        && row.outcome() == EffectLifecyclePhase4LaneOutcome::Verified));
    assert!(rows.iter().any(
        |row| row.lane_kind() == EffectLifecyclePhase4LaneKind::BridgeOracle
            && row.outcome() == EffectLifecyclePhase4LaneOutcome::Verified
    ));
    assert!(rows.iter().any(
        |row| row.lane_kind() == EffectLifecyclePhase4LaneKind::SeededReplay
            && row.outcome() == EffectLifecyclePhase4LaneOutcome::Certified
    ));
}

#[test]
fn phase4_certification_retains_batch_native_and_zero_rediscovery_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_effect_lifecycle_phase4(request_execution);
    let batch = bundle
        .rows()
        .iter()
        .find(|row| row.lane_kind() == EffectLifecyclePhase4LaneKind::BatchExecution)
        .expect("batch lane should be certified");
    let override_denial = bundle
        .rows()
        .iter()
        .find(|row| row.lane_kind() == EffectLifecyclePhase4LaneKind::HostOverrideDenial)
        .expect("host override lane should be certified");

    assert_eq!(batch.counters().batch_lowering_count(), 1);
    assert_eq!(batch.counters().effect_execution_width(), 1);
    assert_eq!(batch.counters().effect_executor_rediscovery_count(), 0);
    assert_eq!(override_denial.counters().effect_execution_width(), 0);
    assert!(!bundle.phase4_bundle_digest().is_empty());
    assert!(!bundle.seeded_bundle_digest().is_empty());
}

#[path = "certification/unified_effect_execution_certification_binds_phase4_seeded_and_p.rs"]
mod unified_effect_execution_certification_binds_phase4_seeded_and_p;

#[test]
fn unified_effect_execution_certification_exposes_named_closeout_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_effect_execution_pipeline(request_execution);

    for output in [
        "query_digest",
        "raw_effect_intent_digest",
        "normalized_effect_intent_digest",
        "effect_family_digest",
        "effect_authority_digest",
        "effect_basis_digest",
        "effect_scope_digest",
        "effect_policy_digest",
        "effect_strategy_digest",
        "effect_eligibility_digest",
        "authority_scoped_effect_plan_digest",
        "lowered_effect_execution_plan_digest",
        "effect_execution_receipt_digest",
        "effect_envelope_digest",
        "relational_effect_authority_digest",
        "bridge_effect_authority_digest",
        "effect_decision_trace_digest",
        "effect_structural_delta_digest",
        "effect_integrity_marker_digest",
        "effect_support_matrix_digest",
        "effect_target_dx_digest",
        "effect_golden_transcript_digest",
        "effect_proof_shape_digest",
        "effect_replay_parity_digest",
        "effect_phase_progression_digest",
        "relational_oracle_digest",
        "bridge_oracle_digest",
        "seeded_sequence_digest",
        "seed_replay_digest",
        "compile_fail_boundary_digest",
        "failure_digest",
        "counter_snapshot",
        "executor_rediscovery_count",
        "batch_lowering_count",
        "batch_basis_reuse_count",
        "authority_reopen_count",
        "effect_normalization_slope_digest",
        "effect_eligibility_slope_digest",
        "effect_lowering_slope_digest",
        "effect_execution_slope_digest",
        "effect_receipt_materialization_slope_digest",
        "effect_envelope_materialization_slope_digest",
        "effect_support_lookup_slope_digest",
    ] {
        assert!(
            bundle.output_digest(output).is_some(),
            "missing required closeout output {output}"
        );
    }
    assert_ne!(
        bundle.output_digest("effect_proof_shape_digest"),
        bundle.output_digest("compile_fail_boundary_digest")
    );
    assert_ne!(
        bundle.output_digest("effect_replay_parity_digest"),
        bundle.output_digest("compile_fail_boundary_digest")
    );
    assert_ne!(
        bundle.output_digest("effect_target_dx_digest"),
        bundle.output_digest("effect_golden_transcript_digest")
    );
    assert_eq!(
        bundle.output_digest("executor_rediscovery_count"),
        Some("0")
    );
    assert_eq!(bundle.output_digest("batch_lowering_count"), Some("1"));
    assert_eq!(bundle.output_digest("batch_basis_reuse_count"), Some("1"));
    assert_eq!(bundle.output_digest("authority_reopen_count"), Some("0"));
}
