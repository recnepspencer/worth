use super::super::support::collect_subscriber_patches_from_head;
use crate::tests::harness::certify::assertions::assert_multi_subscriber_converges;
use crate::tests::harness::fixtures::runtime::RuntimeHarnessMode;
use crate::tests::harness::observe::patch_stream::collect_patch_stream_from_head;
use crate::tests::harness::observe::subscriber_stream::collect_fuzzed_subscriber_views;
use crate::tests::harness::scenario::profiles::CertificationPressureProfile;
use crate::tests::harness::scenario::runner::{run_seeded_scenario, SeededScenarioConfig};
use crate::tests::support::*;

#[test]
fn cdc_certification_subscriber_api_fuzz_matrix_stays_consistent_under_hostile_history_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for (profile, seed_range) in [
        (CertificationPressureProfile::Smoke, 0_u64..2),
        (CertificationPressureProfile::HistoryPressure, 2_u64..4),
    ] {
        for seed in seed_range {
            let world = run_seeded_scenario(
                execution,
                SeededScenarioConfig::hostile_geometry(seed ^ 0xBAD5EED, profile),
            );
            let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
            let patch_head = collect_patch_stream_from_head(&world.runtime, 4096);
            assert_eq!(
                head, patch_head,
                "hostile seed {seed} patch/subscriber drifted"
            );

            let views = collect_fuzzed_subscriber_views(&world.runtime, &head, seed ^ 0xC0DEC0DE);
            assert_multi_subscriber_converges(
                &format!("hostile subscriber api fuzz seed {seed}"),
                &views,
                &head,
            );
        }
    }
}

#[test]
fn cdc_certification_concurrent_branch_merge_pressure_keeps_subscriber_order_stable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = run_seeded_scenario(
        execution,
        SeededScenarioConfig {
            seed: 0xB12A6E55,
            steps: CertificationPressureProfile::HistoryPressure.steps(),
            checkpoint_stride: 8,
            runtime_mode: RuntimeHarnessMode::InMemory(RelationalRuntimeProfile::GeometryKernel),
            relation_pressure: true,
            durable_checkpoint_every: None,
            retention_pass_every: Some(4),
            branch_pressure: true,
            replacement_pressure: true,
        },
    );
    let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
    let patch_head = collect_patch_stream_from_head(&world.runtime, 4096);
    assert_eq!(head, patch_head);

    let views = collect_fuzzed_subscriber_views(&world.runtime, &head, 0xB12A6E55);
    assert_multi_subscriber_converges("branch merge pressure convergence", &views, &head);
}
