use crate::facade::publication::{PublishedAuthoritativePatchEnvelope, SubscriberCheckpoint};
use crate::facade::runtime::RelationalRuntime;
use crate::tests::harness::observe::subscriber_stream::{
    collect_subscriber_patches as collect_subscriber_patches_impl,
    collect_subscriber_patches_from_head as collect_subscriber_patches_from_head_impl,
    expected_patch_suffix_after_checkpoint as expected_patch_suffix_after_checkpoint_impl,
    sampled_checkpoints_from_patches as sampled_checkpoints_from_patches_impl,
};
use crate::tests::harness::scenario::profiles::CertificationPressureProfile;
use crate::tests::harness::scenario::runner::{
    run_seeded_scenario, SeededScenarioConfig, SeededScenarioWorld,
};
use crate::tests::support::*;

pub(super) struct DeterministicCdcScenario {
    pub(super) runtime: RelationalRuntime,
    pub(super) baseline_checkpoint: SubscriberCheckpoint,
}

pub(super) fn run_seeded_cdc_scenario(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: u64,
    steps: usize,
) -> DeterministicCdcScenario {
    let world: SeededScenarioWorld = run_seeded_scenario(
        execution,
        SeededScenarioConfig {
            seed,
            steps,
            checkpoint_stride: CertificationPressureProfile::WindowSplit.steps() / 6,
            runtime_mode: crate::tests::harness::fixtures::runtime::RuntimeHarnessMode::InMemory(
                RelationalRuntimeProfile::GeometryKernel,
            ),
            relation_pressure: false,
            durable_checkpoint_every: None,
            retention_pass_every: None,
            branch_pressure: false,
            replacement_pressure: false,
        },
    );

    DeterministicCdcScenario {
        runtime: world.runtime,
        baseline_checkpoint: world.baseline_checkpoint,
    }
}

pub(super) fn collect_subscriber_patches(
    runtime: &RelationalRuntime,
    checkpoint: SubscriberCheckpoint,
    window_size: usize,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    collect_subscriber_patches_impl(runtime, checkpoint, window_size)
}

pub(super) fn collect_subscriber_patches_from_head(
    runtime: &RelationalRuntime,
    window_size: usize,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    collect_subscriber_patches_from_head_impl(runtime, window_size)
}

pub(super) fn expected_patch_suffix_after_checkpoint(
    patches: &[PublishedAuthoritativePatchEnvelope],
    checkpoint: &SubscriberCheckpoint,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    expected_patch_suffix_after_checkpoint_impl(patches, checkpoint)
}

pub(super) fn sampled_checkpoints_from_patches(
    patches: &[PublishedAuthoritativePatchEnvelope],
    samples: usize,
) -> Vec<SubscriberCheckpoint> {
    sampled_checkpoints_from_patches_impl(patches, samples)
}
