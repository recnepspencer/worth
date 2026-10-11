use super::super::support::{
    collect_subscriber_patches, collect_subscriber_patches_from_head,
    expected_patch_suffix_after_checkpoint, run_seeded_cdc_scenario,
    sampled_checkpoints_from_patches,
};
use crate::facade::publication::SubscriberResumeRequest;
use crate::tests::harness::certify::assertions::{
    assert_multi_subscriber_converges, assert_window_matrix_matches,
};
use crate::tests::harness::observe::subscriber_stream::{
    collect_multi_subscriber_views, random_checkpoints_from_patches,
};
use crate::tests::harness::scenario::profiles::CertificationPressureProfile;
use crate::tests::harness::scenario::runner::{run_seeded_scenario, SeededScenarioConfig};
use crate::tests::support::*;

#[test]
fn cdc_certification_seeded_matrix_is_deterministic_and_resume_exact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in 0_u64..16 {
        let left = run_seeded_cdc_scenario(execution, seed, 96);
        let right = run_seeded_cdc_scenario(execution, seed, 96);

        let full_left =
            collect_subscriber_patches(&left.runtime, left.baseline_checkpoint.clone(), 512);
        let full_from_head = collect_subscriber_patches_from_head(&left.runtime, 512);
        let full_right =
            collect_subscriber_patches(&right.runtime, right.baseline_checkpoint.clone(), 512);
        assert_eq!(full_left, full_right, "seed {seed} diverged");

        let window_matrix = [1_usize, 2, 3, 5, 8, 13]
            .into_iter()
            .map(|window_size| {
                (
                    window_size,
                    collect_subscriber_patches(
                        &left.runtime,
                        left.baseline_checkpoint.clone(),
                        window_size,
                    ),
                )
            })
            .collect::<Vec<_>>();
        assert_window_matrix_matches(
            &format!("seed {seed} baseline window matrix"),
            &full_left,
            &window_matrix,
        );

        let checkpoints = sampled_checkpoints_from_patches(&full_left, 6);
        let views = collect_multi_subscriber_views(&left.runtime, &checkpoints, &[1, 2, 4, 7]);
        assert_multi_subscriber_converges(
            &format!("seed {seed} multi-subscriber convergence"),
            &views,
            &full_from_head,
        );

        for checkpoint in checkpoints {
            let expected = expected_patch_suffix_after_checkpoint(&full_left, &checkpoint);
            let resumed = collect_subscriber_patches(&left.runtime, checkpoint.clone(), 4);
            assert_eq!(
                resumed,
                expected,
                "seed {seed} resume drifted after {:?}",
                checkpoint.position()
            );
        }
    }
}

#[test]
#[ignore = "scheduled hostile CDC certification; run explicitly in the scheduled relational lane"]
fn cdc_certification_thousand_step_random_resume_matrix_converges() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in 0_u64..3 {
        let world = run_seeded_scenario(
            execution,
            SeededScenarioConfig::geometry_kernel(seed, CertificationPressureProfile::ThousandStep),
        );
        assert!(
            !world.trace.operations.is_empty(),
            "seed {seed} produced no operations"
        );
        assert_eq!(world.trace.seed, seed);

        let full_from_head = collect_subscriber_patches_from_head(&world.runtime, 2048);
        let checkpoints = random_checkpoints_from_patches(&full_from_head, seed ^ 0xA5A5_A5A5, 12);
        let views = collect_multi_subscriber_views(
            &world.runtime,
            &checkpoints,
            CertificationPressureProfile::ThousandStep.default_windows(),
        );

        assert_multi_subscriber_converges(
            &format!(
                "seed {} thousand-step random resume matrix over {} checkpoints",
                seed,
                world.checkpoints.len()
            ),
            &views,
            &full_from_head,
        );
    }
}

#[test]
fn cdc_certification_explicit_dependency_graph_resume_is_exact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let source = create_entity_in_partition(execution, &runtime, "source-a", PartitionId(7));
    let target = create_entity_in_partition(execution, &runtime, "target-b", PartitionId(11));
    let _dependency = create_relation_in_partition(
        execution,
        &runtime,
        source,
        target,
        "depends-on",
        PartitionId(29),
    );
    let baseline_checkpoint = checkpoint_for_schema_version(
        runtime.publication().latest_patch().unwrap().position,
        SchemaVersionId(1),
    );

    for step in 0..48 {
        update_entity_and_release_snapshot(
            execution,
            &runtime,
            target,
            &format!("target-b-{step}"),
        );
        if step % 4 == 0 {
            update_entity_and_release_snapshot(
                execution,
                &runtime,
                source,
                &format!("source-a-{step}"),
            );
        }
        if step % 6 == 0 {
            let _ = create_entity_in_partition(
                execution,
                &runtime,
                &format!("churn-{step}"),
                match step % 3 {
                    0 => PartitionId(7),
                    1 => PartitionId(11),
                    _ => PartitionId(31),
                },
            );
        }
    }

    let full = collect_subscriber_patches(&runtime, baseline_checkpoint.clone(), 512);
    let dependency_checkpoints = random_checkpoints_from_patches(&full, 0xDEADBEEF, 10);
    let views = collect_multi_subscriber_views(&runtime, &dependency_checkpoints, &[1, 2, 3, 5, 8]);
    assert_multi_subscriber_converges(
        "explicit dependency graph resume exactness",
        &views,
        &collect_subscriber_patches_from_head(&runtime, 512),
    );
}

#[test]
#[ignore = "scheduled hostile CDC certification; run explicitly in the scheduled relational lane"]
fn cdc_certification_rewrite_storm_preserves_exact_suffix_under_tiny_windows() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let entity = create_entity_in_partition(execution, &runtime, "rewrite-storm-0", PartitionId(7));
    let profile = CertificationPressureProfile::RewriteStorm;
    let baseline_checkpoint = checkpoint_for_schema_version(
        runtime.publication().latest_patch().unwrap().position,
        SchemaVersionId(1),
    );

    for step in 1..=profile.steps() {
        update_entity_and_release_snapshot(
            execution,
            &runtime,
            entity,
            &format!("rewrite-storm-{step}"),
        );
        if step % 16 == 0 {
            let partition = match step % 4 {
                0 => PartitionId(7),
                1 => PartitionId(11),
                2 => PartitionId(29),
                _ => PartitionId(31),
            };
            let _ = create_entity_in_partition(
                execution,
                &runtime,
                &format!("storm-churn-{step}"),
                partition,
            );
        }
    }

    let head = collect_subscriber_patches_from_head(&runtime, 8192);
    let resumed = collect_subscriber_patches(&runtime, baseline_checkpoint.clone(), 1);
    let expected = expected_patch_suffix_after_checkpoint(&head, &baseline_checkpoint);
    assert_eq!(resumed, expected);

    let checkpoint_samples = random_checkpoints_from_patches(&head, 0x515E515E, 24);
    let views =
        collect_multi_subscriber_views(&runtime, &checkpoint_samples, profile.default_windows());
    assert_multi_subscriber_converges("rewrite storm tiny-window convergence", &views, &head);
}

#[test]
fn cdc_certification_restart_loops_do_not_leak_resume_state_across_sessions() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = run_seeded_scenario(
        execution,
        SeededScenarioConfig::hostile_geometry(
            0xABCD1234,
            CertificationPressureProfile::HistoryPressure,
        ),
    );
    let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
    let checkpoints = random_checkpoints_from_patches(&head, 0xD15C0C7, 20);

    for checkpoint in checkpoints {
        let expected = expected_patch_suffix_after_checkpoint(&head, &checkpoint);
        let mut session_checkpoint = checkpoint.clone();
        let mut stitched = Vec::new();

        for cycle in 0..1024 {
            let batch = world
                .runtime
                .publication()
                .read_subscriber_stream(SubscriberResumeRequest::resume_after(
                    session_checkpoint.clone(),
                    (cycle % 3) + 1,
                ))
                .unwrap();
            if batch.patches.is_empty() {
                break;
            }
            stitched.extend(batch.patches.clone());
            let Some(next_checkpoint) = batch.next_checkpoint else {
                break;
            };
            if next_checkpoint == session_checkpoint {
                break;
            }
            session_checkpoint = next_checkpoint;
        }

        assert_eq!(stitched, expected);
    }
}
