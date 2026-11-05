//! The isolated recovery owner applies each recorded mutation policy during replay.
use crate::runtime::RelationalRuntime;
impl RelationalRuntime {
    pub(crate) fn restore_source_replay_configuration(
        &mut self,
        policy: &crate::config::data::MutationConfig,
        schema: &crate::schema::data::RelationalSchemaRegistry,
    ) {
        let binding = self.configuration_binding();
        {
            let mut state = binding.initial_installation();
            let mut config = state.config.as_ref().clone();
            config.storage.cascade_delete_policy = policy.cascade_delete_policy;
            config.storage.adjacency_policy = policy.adjacency_policy.clone();
            config.storage.cross_context_policy = policy.cross_context_policy;
            if config.schema.registry != *schema {
                let rebuilt = super::rebuilt_schema_contract_runtime(
                    schema,
                    state
                        .schema_contract_runtime
                        .custom_invariant_registries
                        .clone(),
                    state.schema_contract_runtime.custom_invariant_generation,
                    state
                        .schema_contract_runtime
                        .initial_custom_invariants_sealed,
                    crate::schema::SchemaCompilationControl::Configuration,
                )
                .expect("request-free replay configuration has no execution refusal");
                config.schema.registry = schema.clone();
                state.schema_contract_runtime = std::sync::Arc::new(rebuilt);
            }
            state.config = std::sync::Arc::new(config);
        }
        self.reconfigure(|_| {});
    }
}
