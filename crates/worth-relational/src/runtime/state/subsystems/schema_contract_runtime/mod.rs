use crate::runtime::state::subsystems::RuntimeSubsystem;
use crate::schema::{lower_aspect_plans, SchemaCompilationControl};
use crate::validation::FrozenCustomInvariantRegistry;
mod native_registrations;
use native_registrations::NativeRelationIntegrityInventory;

#[derive(Debug, Clone, Default)]
pub(crate) struct SchemaContractRuntimeSubsystem {
    pub(crate) aspect_contract_plans: crate::branch::RetainedRelationalAspectPlans,
    pub(crate) relation_integrity_registrations: std::sync::Arc<NativeRelationIntegrityInventory>,
    pub(crate) custom_invariant_registries: FrozenCustomInvariantRegistry,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) initial_custom_invariants_sealed: bool,
    pub(in crate::runtime) home_configuration_backing: Option<
        std::sync::Arc<
            crate::runtime::home_program_transition::configuration::HomeConfigurationBacking,
        >,
    >,
}

impl RuntimeSubsystem for SchemaContractRuntimeSubsystem {
    type Config = crate::config::data::RelationalRuntimeConfig;

    fn new(config: &Self::Config) -> Self {
        Self::for_schema(
            &config.schema.registry,
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

impl SchemaContractRuntimeSubsystem {
    /// Initial installation and a Home target lower the same complete inventory.
    /// The supplied aspect plans are the exact selected root schema's plans.
    pub(crate) fn for_schema(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        aspect_contract_plans: crate::branch::RetainedRelationalAspectPlans,
        custom_invariant_registries: FrozenCustomInvariantRegistry,
        custom_invariant_generation: u64,
        initial_custom_invariants_sealed: bool,
    ) -> Self {
        Self::for_schema_compilation(
            registry,
            aspect_contract_plans,
            custom_invariant_registries,
            custom_invariant_generation,
            initial_custom_invariants_sealed,
            SchemaCompilationControl::Configuration,
        )
        .expect("configuration native registration compilation has no request refusal")
    }
    pub(crate) fn for_schema_compilation(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        aspect_contract_plans: crate::branch::RetainedRelationalAspectPlans,
        custom_invariant_registries: FrozenCustomInvariantRegistry,
        custom_invariant_generation: u64,
        initial_custom_invariants_sealed: bool,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        let registrations = NativeRelationIntegrityInventory::compile(registry, control)?;
        Ok(Self {
            aspect_contract_plans,
            relation_integrity_registrations: registrations,
            custom_invariant_registries,
            custom_invariant_generation,
            initial_custom_invariants_sealed,
            home_configuration_backing: None,
        })
    }
}
