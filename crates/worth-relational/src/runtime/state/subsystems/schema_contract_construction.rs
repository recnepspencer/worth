//! Root-side construction of the lowered executable schema inventory.
use super::schema_contract_runtime::SchemaContractRuntimeSubsystem;
use super::RuntimeSubsystem;
use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::schema::lower_aspect_plans;
use crate::validation::custom_registry::FrozenCustomInvariantRegistry;

impl RuntimeSubsystem for SchemaContractRuntimeSubsystem {
    type Config = RelationalRuntimeConfig;

    fn new(config: &Self::Config) -> Self {
        Self::for_schema(
            &config.schema.registry,
            &config.schema.invariant_catalog.registrations,
            crate::branch::RetainedRelationalAspectPlans::from_configuration(lower_aspect_plans(
                &config.schema.registry,
            )),
            FrozenCustomInvariantRegistry::default(),
            0,
            false,
        )
    }

    fn fork(&self) -> Self {
        self.clone()
    }
}
