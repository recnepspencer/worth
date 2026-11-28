use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::events::EventBus;

use super::super::runtime_state::SignalRuntime;
use super::{Present, SignalRuntimeBuilder};

impl<D, I, E, Ctx, T> SignalRuntimeBuilder<Present, Present, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn build_validated(
        self,
    ) -> Result<SignalRuntime<D, I, E, Ctx, T>, crate::data::error::SignalError> {
        let requested_policy = self.runtime_policy;
        let installed_policy = crate::runtime_policy::compile_signal_runtime_policy(
            crate::runtime_policy::SignalRuntimePolicyRequest::new(requested_policy),
        )
        .map_err(|_| crate::data::error::SignalError::InvalidInput {
            message: "runtime policy admission failed".to_string(),
            context: Some("SignalRuntimeBuilder::build_validated".to_string()),
        })?;
        let checkpoint = CheckpointRuntime::new(self.checkpoint_policy);
        let event_bus = EventBus::new();
        let mut runtime =
            SignalRuntime::new(self.graph, self.schema_registry, checkpoint, event_bus);
        runtime
            .branches
            .set_maximum_stored_snapshots(self.maximum_stored_branch_snapshots);

        runtime
            .resource
            .set_policy_registry(self.resource_policy_registry);
        runtime.set_fallback_comparator(self.fallback_comparator);
        runtime
            .graph
            .install_compiled_runtime_policy(requested_policy, installed_policy);
        for policy in self.tier_policies {
            runtime.set_tier_policy(policy);
        }
        runtime.validate_schema_bindings()?;

        Ok(runtime)
    }

    pub fn build(self) -> SignalRuntime<D, I, E, Ctx, T> {
        self.build_validated()
            .expect("signal runtime schema bindings must match the effective frozen registries")
    }
}
