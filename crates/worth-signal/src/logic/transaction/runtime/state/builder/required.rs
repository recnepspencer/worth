use std::marker::PhantomData;

use crate::data::checkpoint::CheckpointBarrier;
use crate::data::checkpoint_policy::CheckpointPolicy;
use crate::data::comparator::VersionComparatorPolicy;

use super::{Present, SignalRuntimeBuilder};

impl<CheckpointState, ComparatorState, D, I, E, Ctx, T>
    SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Set a simple checkpoint barrier policy.
    pub fn checkpoint_barrier(
        self,
        barrier: CheckpointBarrier,
    ) -> SignalRuntimeBuilder<Present, ComparatorState, D, I, E, Ctx, T> {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: CheckpointPolicy::new(barrier),
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    /// Set the full checkpoint policy.
    pub fn checkpoint_policy(
        self,
        policy: CheckpointPolicy<D>,
    ) -> SignalRuntimeBuilder<Present, ComparatorState, D, I, E, Ctx, T> {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    /// Adjust the current checkpoint setup in one place.
    pub fn adjust_checkpoints<F>(
        mut self,
        adjust: F,
    ) -> SignalRuntimeBuilder<Present, ComparatorState, D, I, E, Ctx, T>
    where
        F: FnOnce(&mut CheckpointPolicy<D>),
    {
        adjust(&mut self.checkpoint_policy);
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    /// Set the fallback comparator used when a node or tier does not provide one.
    pub fn fallback_comparator(
        self,
        comparator: VersionComparatorPolicy,
    ) -> SignalRuntimeBuilder<CheckpointState, Present, D, I, E, Ctx, T> {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    /// Adjust the fallback comparator without restating the whole value.
    pub fn adjust_fallback_comparator<F>(
        mut self,
        adjust: F,
    ) -> SignalRuntimeBuilder<CheckpointState, Present, D, I, E, Ctx, T>
    where
        F: FnOnce(VersionComparatorPolicy) -> VersionComparatorPolicy,
    {
        self.fallback_comparator = adjust(self.fallback_comparator);
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    pub fn with_kernel_defaults(self) -> SignalRuntimeBuilder<Present, Present, D, I, E, Ctx, T> {
        self.checkpoint_barrier(CheckpointBarrier::PerOperation)
            .fallback_comparator(VersionComparatorPolicy::Exact)
    }

    pub fn with_events<E2>(
        self,
    ) -> SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I, E2, Ctx, T> {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    pub fn with_domains<D2>(
        self,
    ) -> SignalRuntimeBuilder<CheckpointState, ComparatorState, D2, I, E, Ctx, T>
    where
        D2: Copy + Ord + std::fmt::Debug + 'static,
    {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: CheckpointPolicy::new(self.checkpoint_policy.barrier_for_default()),
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_impacts<I2>(
        self,
    ) -> SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I2, E, Ctx, T>
    where
        I2: Copy + Ord,
    {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }

    pub fn with_tiers<T2>(
        self,
    ) -> SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I, E, Ctx, T2>
    where
        T2: Copy + Ord,
    {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_context<Ctx2>(
        self,
    ) -> SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I, E, Ctx2, T> {
        SignalRuntimeBuilder {
            graph: self.graph,
            schema_registry: self.schema_registry,

            resource_policy_registry: self.resource_policy_registry,
            checkpoint_policy: self.checkpoint_policy,
            fallback_comparator: self.fallback_comparator,
            runtime_policy: self.runtime_policy,
            maximum_stored_branch_snapshots: self.maximum_stored_branch_snapshots,
            tier_policies: self.tier_policies,
            _marker: PhantomData,
        }
    }
}
