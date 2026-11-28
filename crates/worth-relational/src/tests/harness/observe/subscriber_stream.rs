use crate::facade::publication::{
    PublishedAuthoritativePatchEnvelope, SubscriberCheckpoint, SubscriberResumeRequest,
};
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{checkpoint_for_schema_version, SchemaVersionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SubscriberView {
    pub(crate) checkpoint: Option<SubscriberCheckpoint>,
    pub(crate) window_size: usize,
    pub(crate) patches: Vec<PublishedAuthoritativePatchEnvelope>,
}

pub(crate) fn collect_subscriber_patches(
    runtime: &RelationalRuntime,
    checkpoint: SubscriberCheckpoint,
    window_size: usize,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    collect_subscriber_patches_from_request(
        runtime,
        SubscriberResumeRequest::resume_after(checkpoint, window_size),
    )
}

pub(crate) fn collect_subscriber_patches_from_head(
    runtime: &RelationalRuntime,
    window_size: usize,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    collect_subscriber_patches_from_request(
        runtime,
        SubscriberResumeRequest::from_head(window_size),
    )
}

pub(crate) fn expected_patch_suffix_after_checkpoint(
    patches: &[PublishedAuthoritativePatchEnvelope],
    checkpoint: &SubscriberCheckpoint,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    patches
        .iter()
        .filter(|patch| patch.position.0 > checkpoint.position().0)
        .cloned()
        .collect()
}

pub(crate) fn sampled_checkpoints_from_patches(
    patches: &[PublishedAuthoritativePatchEnvelope],
    samples: usize,
) -> Vec<SubscriberCheckpoint> {
    if patches.is_empty() || samples == 0 {
        return Vec::new();
    }

    let stride = (patches.len() / samples.max(1)).max(1);
    patches
        .iter()
        .enumerate()
        .skip(stride.saturating_sub(1))
        .step_by(stride)
        .map(|(_, patch)| checkpoint_for_schema_version(patch.position, SchemaVersionId(1)))
        .collect()
}

pub(crate) fn random_checkpoints_from_patches(
    patches: &[PublishedAuthoritativePatchEnvelope],
    seed: u64,
    samples: usize,
) -> Vec<SubscriberCheckpoint> {
    if patches.is_empty() || samples == 0 {
        return Vec::new();
    }

    let mut rng = worth_dst::Seed::new(seed).rng();
    let mut checkpoints = Vec::new();
    for _ in 0..samples.min(patches.len()) {
        let index = rng.below(patches.len());
        let checkpoint = checkpoint_for_schema_version(patches[index].position, SchemaVersionId(1));
        if !checkpoints.contains(&checkpoint) {
            checkpoints.push(checkpoint);
        }
    }
    checkpoints.sort_by_key(|checkpoint| checkpoint.position().0);
    checkpoints
}

pub(crate) fn collect_multi_subscriber_views(
    runtime: &RelationalRuntime,
    checkpoints: &[SubscriberCheckpoint],
    window_sizes: &[usize],
) -> Vec<SubscriberView> {
    let mut views = Vec::new();
    for window_size in window_sizes {
        views.push(SubscriberView {
            checkpoint: None,
            window_size: *window_size,
            patches: collect_subscriber_patches_from_head(runtime, *window_size),
        });
        for checkpoint in checkpoints {
            views.push(SubscriberView {
                checkpoint: Some(checkpoint.clone()),
                window_size: *window_size,
                patches: collect_subscriber_patches(runtime, checkpoint.clone(), *window_size),
            });
        }
    }
    views
}

pub(crate) fn collect_fuzzed_subscriber_views(
    runtime: &RelationalRuntime,
    patches_from_head: &[PublishedAuthoritativePatchEnvelope],
    seed: u64,
) -> Vec<SubscriberView> {
    let checkpoints = random_checkpoints_from_patches(patches_from_head, seed ^ 0xA11CE5EED, 16);
    let windows = fuzz_window_sizes(seed);
    collect_multi_subscriber_views(runtime, &checkpoints, &windows)
}

pub(crate) fn fuzz_window_sizes(seed: u64) -> Vec<usize> {
    const CANDIDATES: [usize; 12] = [1, 2, 3, 4, 5, 7, 8, 13, 16, 21, 34, 55];

    let mut state = seed
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(0xD1B54A32D192ED03);
    let mut windows = vec![1, 2, 3, 5, 8];
    for candidate in CANDIDATES {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if state & 1 == 0 && !windows.contains(&candidate) {
            windows.push(candidate);
        }
    }
    windows.sort_unstable();
    windows.dedup();
    windows
}

fn collect_subscriber_patches_from_request(
    runtime: &RelationalRuntime,
    mut request: SubscriberResumeRequest,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    let mut collected = Vec::new();

    loop {
        let batch = runtime
            .publication()
            .read_subscriber_stream(request.clone())
            .unwrap();

        if batch.patches.is_empty() {
            break;
        }

        collected.extend(batch.patches.clone());
        let Some(next_checkpoint) = batch.next_checkpoint else {
            break;
        };

        if batch.resumed_from.as_ref() == Some(&next_checkpoint) {
            break;
        }
        request = SubscriberResumeRequest::resume_after(next_checkpoint, request.max_commits());
    }

    collected
}
