use crate::data::resource::FrozenResourcePolicyRegistry;
use crate::data::tier::TierPolicy;
use crate::runtime_policy::SignalRuntimePolicy;
use crate::schema::data::SignalSchemaRegistry;

use super::SignalRuntimeBuilder;

impl<CheckpointState, ComparatorState, D, I, E, Ctx, T>
    SignalRuntimeBuilder<CheckpointState, ComparatorState, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn runtime_policy(mut self, runtime_policy: SignalRuntimePolicy) -> Self {
        self.runtime_policy = runtime_policy;
        self
    }

    /// Bound the total heavy branch snapshot states retained by this runtime.
    pub fn maximum_stored_branch_snapshots(mut self, maximum: usize) -> Self {
        self.maximum_stored_branch_snapshots = maximum;
        self
    }

    pub fn adjust_runtime_policy<F>(mut self, adjust: F) -> Self
    where
        F: FnOnce(SignalRuntimePolicy) -> SignalRuntimePolicy,
    {
        self.runtime_policy = adjust(self.runtime_policy);
        self
    }

    pub fn development_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::development();
        self
    }

    pub fn operational_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::operational();
        self
    }

    pub fn forensic_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::forensic();
        self
    }

    pub fn web_development_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::web_development();
        self
    }

    pub fn fintech_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::fintech();
        self
    }

    pub fn kernel_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::kernel();
        self
    }

    pub fn game_engine_policy(mut self) -> Self {
        self.runtime_policy = SignalRuntimePolicy::game_engine();
        self
    }

    pub fn tier_policy(mut self, policy: TierPolicy<T>) -> Self {
        self.tier_policies.push(policy);
        self
    }

    pub fn schema_registry(mut self, schema_registry: SignalSchemaRegistry) -> Self {
        self.schema_registry = schema_registry;
        self
    }

    pub fn resource_policy_registry(
        mut self,
        resource_policy_registry: FrozenResourcePolicyRegistry,
    ) -> Self {
        self.resource_policy_registry = resource_policy_registry;
        self
    }
}
