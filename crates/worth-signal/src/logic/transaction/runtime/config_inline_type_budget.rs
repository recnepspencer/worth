//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalRuntimeConfig<()>>(
            "crates/worth-signal/src/logic/transaction/runtime/config.rs",
            "SignalRuntimeConfig",
            128 * 1024,
        );
        owner.field::<NodeMetaStore<()>>("node_meta");
        owner.field::<TierPolicyTable<()>>("tier_policies");
        owner.field::<VersionComparatorPolicy>("fallback_comparator");
        owner.field::<RuntimeKeyRegistry>("key_registry");
        owner.field::<crate::data::persistent_ord_map::PersistentOrdMap<
            RuntimeStringId,
            ComputationRegistration<()>,
        >>("computations");
        owner.field::<crate::data::persistent_ord_map::PersistentOrdMap<
            (RuntimeStringId, RuntimeStringId),
            NodeId,
        >>("keyed_nodes");
        owner.field::<crate::data::persistent_ord_map::PersistentOrdMap<
            (RuntimeStringId, RuntimeStringId, RuntimeStringId),
            NodeEvaluationResult,
        >>("memo_cache");
        owner.field::<Option<TemporalDuration>>("resource_runtime_deadline");
    }
}
