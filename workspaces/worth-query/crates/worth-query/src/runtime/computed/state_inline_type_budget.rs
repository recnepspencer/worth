//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryComputedDependencyIndex>(
            "workspaces/worth-query/crates/worth-query/src/runtime/computed/state.rs",
            "WorthQueryComputedDependencyIndex",
            128 * 1024,
        );
        owner.field::<BTreeMap<
            WorthQueryLiveArtifactTarget,
            BTreeSet<WorthQueryDerivedMaterializationTarget>,
        >>("live_to_computed");
        owner.field::<BTreeMap<
            WorthQueryDerivedMaterializationTarget,
            BTreeSet<WorthQueryDerivedMaterializationTarget>,
        >>("computed_to_dependents");
        owner.field::<BTreeSet<WorthQueryDerivedMaterializationTarget>>(
            "unscoped_authoritative_computed",
        );
    }
}
