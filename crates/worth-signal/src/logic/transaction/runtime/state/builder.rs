use std::marker::PhantomData;

use crate::data::checkpoint::CheckpointBarrier;
use crate::data::checkpoint_policy::CheckpointPolicy;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::graph::SignalGraph;
use crate::data::resource::FrozenResourcePolicyRegistry;
use crate::data::tier::TierPolicy;
use crate::runtime_policy::SignalRuntimePolicy;
use crate::schema::data::SignalSchemaRegistry;

use super::branching::DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS;
use super::merge::{
    FrozenAspectMergePolicyRegistry, FrozenConflictIsolationRegistry, FrozenConflictPolicyRegistry,
    FrozenDeletionPolicyRegistry, FrozenIdentityMatcherRegistry, FrozenMergeBaseStrategyRegistry,
    FrozenMergeStrategyRegistry, FrozenSourceOnlyPolicyRegistry,
};

mod assembly;
mod policy;
mod required;

pub struct Missing;
pub struct Present;

/// Builder for `SignalRuntime`.
///
/// Required runtime capabilities are tracked by the first two type parameters;
/// optional policy registration remains ordinary builder state.
pub struct SignalRuntimeBuilder<
    CheckpointState = Missing,
    ComparatorState = Missing,
    D = (),
    I = (),
    E = (),
    Ctx = (),
    T = (),
> where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    graph: Box<SignalGraph>,
    schema_registry: SignalSchemaRegistry,
    merge_strategy_registry: FrozenMergeStrategyRegistry,
    merge_base_strategy_registry: FrozenMergeBaseStrategyRegistry,
    aspect_merge_policy_registry: FrozenAspectMergePolicyRegistry,
    conflict_isolation_registry: FrozenConflictIsolationRegistry,
    conflict_policy_registry: FrozenConflictPolicyRegistry,
    identity_matcher_registry: FrozenIdentityMatcherRegistry,
    source_only_policy_registry: FrozenSourceOnlyPolicyRegistry,
    deletion_policy_registry: FrozenDeletionPolicyRegistry,
    resource_policy_registry: FrozenResourcePolicyRegistry,
    checkpoint_policy: CheckpointPolicy<D>,
    fallback_comparator: VersionComparatorPolicy,
    runtime_policy: SignalRuntimePolicy,
    maximum_stored_branch_snapshots: usize,
    tier_policies: Vec<TierPolicy<T>>,
    _marker: PhantomData<fn(CheckpointState, ComparatorState, I, E, Ctx, T)>,
}

impl<CheckpointState, ComparatorState, D, I, E, Ctx, T>
    SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn new(graph: SignalGraph) -> Self {
        Self::new_boxed(Box::new(graph))
    }

    pub(super) fn new_boxed(graph: Box<SignalGraph>) -> Self {
        Self {
            graph,
            schema_registry: SignalSchemaRegistry::default(),
            merge_strategy_registry: FrozenMergeStrategyRegistry::built_in(),
            merge_base_strategy_registry: FrozenMergeBaseStrategyRegistry::built_in(),
            aspect_merge_policy_registry: FrozenAspectMergePolicyRegistry::built_in(),
            conflict_isolation_registry: FrozenConflictIsolationRegistry::built_in(),
            conflict_policy_registry: FrozenConflictPolicyRegistry::built_in(),
            identity_matcher_registry: FrozenIdentityMatcherRegistry::built_in(),
            source_only_policy_registry: FrozenSourceOnlyPolicyRegistry::built_in(),
            deletion_policy_registry: FrozenDeletionPolicyRegistry::built_in(),
            resource_policy_registry: FrozenResourcePolicyRegistry::built_in(),
            checkpoint_policy: CheckpointPolicy::new(CheckpointBarrier::PerOperation),
            fallback_comparator: VersionComparatorPolicy::Exact,
            runtime_policy: SignalRuntimePolicy::default(),
            maximum_stored_branch_snapshots: DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS,
            tier_policies: Vec::new(),
            _marker: PhantomData,
        }
    }
}
