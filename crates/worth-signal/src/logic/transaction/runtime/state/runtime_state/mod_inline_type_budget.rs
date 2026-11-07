//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalRuntime<(), (), (), (), ()>>(
            "crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/mod.rs",
            "SignalRuntime",
            128 * 1024,
        );
        owner.field::<SignalRuntimeConfig<()>>("config");
        owner.field::<Box<SignalGraph>>("graph");
        owner.field::<SignalSchemaRegistry>("schema_registry");
        owner.field::<FrozenMergeStrategyRegistry>("merge_strategy_registry");
        owner.field::<FrozenMergeBaseStrategyRegistry>("merge_base_strategy_registry");
        owner.field::<FrozenAspectMergePolicyRegistry>("aspect_merge_policy_registry");
        owner.field::<FrozenConflictIsolationRegistry>("conflict_isolation_registry");
        owner.field::<FrozenConflictPolicyRegistry>("conflict_policy_registry");
        owner.field::<FrozenIdentityMatcherRegistry>("identity_matcher_registry");
        owner.field::<FrozenSourceOnlyPolicyRegistry>("source_only_policy_registry");
        owner.field::<FrozenDeletionPolicyRegistry>("deletion_policy_registry");
        owner.field::<CheckpointRuntime<(), ()>>("checkpoint");
        owner.field::<EventBus<(), (), ()>>("event_bus");
        owner.field::<RuntimeObservationRegistry<(), (), (), (), ()>>("observations");
        owner.field::<ResourceRuntimeState>("resource");
        owner.field::<TemporalRuntimeState>("temporal");
        owner.field::<RuntimeTelemetry>("telemetry");
        owner.field::<BranchManager<(), (), ()>>("branches");
        owner.field::<crate::branch::SignalBranchBasisRegistry>("basis_registry");
        owner.field::<crate::branch::owner_services::SignalOwnerRoot<(), (), ()>>("owner_services");
    }
    super::branch_state_capture::inline_type_budget::collect(census);
    super::branch_transfer::inline_type_budget::collect(census);
}
