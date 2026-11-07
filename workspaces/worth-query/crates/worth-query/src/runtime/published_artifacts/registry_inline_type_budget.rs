//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPublishedArtifactRegistry>(
            "workspaces/worth-query/crates/worth-query/src/runtime/published_artifacts/registry.rs",
            "WorthQueryPublishedArtifactRegistry",
            128 * 1024,
        );
        owner.field::<Arc<ArcSwap<WorthQueryPublishedArtifactGenerations>>>("generations");
        owner.field::<WorthQueryPublishedArtifactCounters>("counters");
    }
}
