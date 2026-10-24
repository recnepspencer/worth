use super::super::support::{
    collect_subscriber_patches, collect_subscriber_patches_from_head,
    expected_patch_suffix_after_checkpoint,
};
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::publication::{SubscriberRecoverySource, SubscriberResumeRequest};
use crate::tests::harness::certify::assertions::assert_multi_subscriber_converges;
use crate::tests::harness::fixtures::runtime::RuntimeHarnessMode;
use crate::tests::harness::observe::patch_stream::collect_patch_stream_from_head;
use crate::tests::harness::observe::subscriber_stream::{
    collect_multi_subscriber_views, random_checkpoints_from_patches,
};
use crate::tests::harness::scenario::profiles::CertificationPressureProfile;
use crate::tests::harness::scenario::runner::{
    build_property_runtime, run_seeded_scenario, SeededScenarioConfig,
};
use crate::tests::support::*;

#[test]
fn cdc_certification_durable_recovery_matches_head_and_midstream_consumers() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    for name in ["a", "b", "c", "d", "e", "f"] {
        let _ = create_entity_outcome(execution, &runtime, name);
        if ["b", "d"].contains(&name) {
            crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
        }
    }

    let full_head = collect_subscriber_patches_from_head(&runtime, 128);
    let mid_checkpoint = checkpoint_for_schema_version(PatchStreamPosition(3), SchemaVersionId(1));
    let expected_mid = expected_patch_suffix_after_checkpoint(&full_head, &mid_checkpoint);

    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(1)));
    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(2)));
    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(3)));

    let durable_mid = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(mid_checkpoint, 2))
        .unwrap();
    assert_eq!(
        durable_mid.recovery_decision.source,
        SubscriberRecoverySource::DurableCanonicalRecovery
    );

    let durable_mid_stitched = collect_subscriber_patches(
        &runtime,
        checkpoint_for_schema_version(PatchStreamPosition(3), SchemaVersionId(1)),
        2,
    );
    assert_eq!(durable_mid_stitched, expected_mid);

    let recovery_plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, recovery_plan)
        .unwrap();
    let recovered_patch_batch = recovered
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(PatchStreamPosition(3)),
            max_commits: 32,
        })
        .unwrap();
    assert_eq!(durable_mid_stitched, recovered_patch_batch.patches);
}

#[test]
fn cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in 0_u64..3 {
        let world = run_seeded_scenario(
            execution,
            SeededScenarioConfig::persisted_geometry(
                seed,
                CertificationPressureProfile::HistoryPressure,
            ),
        );
        let full_from_head = collect_subscriber_patches_from_head(&world.runtime, 4096);
        let checkpoints = random_checkpoints_from_patches(&full_from_head, seed ^ 0x55AA55AA, 10);
        let views = collect_multi_subscriber_views(&world.runtime, &checkpoints, &[1, 2, 4, 8, 16]);
        assert_multi_subscriber_converges(
            &format!("persisted seed {seed} pre-recovery convergence"),
            &views,
            &full_from_head,
        );

        let recovery_plan = crate::tests::support::recovery_plan_for(
            execution,
            &world.runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
        let mut recovered = build_property_runtime(RuntimeHarnessMode::Persisted);
        recovered
            .durability_recovery()
            .recover(execution, recovery_plan)
            .unwrap();
        let recovered_patch_stream = collect_patch_stream_from_head(&recovered, 4096);
        assert_eq!(
            recovered_patch_stream, full_from_head,
            "persisted seed {seed} drifted after recovery"
        );
    }
}

#[test]
fn cdc_certification_retention_truncation_recovers_exact_suffix_from_old_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = run_seeded_scenario(
        execution,
        SeededScenarioConfig::persisted_geometry(
            0xFACE515E,
            CertificationPressureProfile::HistoryPressure,
        ),
    );
    let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
    let old_checkpoint = random_checkpoints_from_patches(&head, 0x0DCAFE, 1)
        .into_iter()
        .next()
        .unwrap_or_else(|| world.baseline_checkpoint.clone());

    for _ in 0..16 {
        let _ = world.runtime.retention().run_pass();
    }

    let resumed = collect_subscriber_patches(&world.runtime, old_checkpoint.clone(), 3);
    let expected = expected_patch_suffix_after_checkpoint(&head, &old_checkpoint);
    assert_eq!(resumed, expected);
}
