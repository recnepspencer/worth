//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryEffectIndex>(
            "workspaces/worth-query/crates/worth-query/src/runtime/effect/registry.rs",
            "WorthQueryEffectIndex",
            128 * 1024,
        );
        owner.field::<BTreeMap<WorthQueryLiveArtifactTarget, BTreeSet<WorthQueryEffectTarget>>>(
            "live_to_effects",
        );
        owner.field::<BTreeMap<WorthQueryDerivedMaterializationTarget, BTreeSet<WorthQueryEffectTarget>>>("computed_to_effects");
    }
}
