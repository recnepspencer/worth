use crate::runtime::state::subsystems::RuntimeSubsystem;
use crate::schema::data::AspectContractPlanCatalog;
#[cfg(test)]
use crate::schema::data::RelationIntegrityPlanCatalog;
use crate::schema::{lower_aspect_plans, lower_relation_integrity_plans};
use crate::validation::data::InvariantRegistration;
use crate::validation::FrozenCustomInvariantRegistry;

#[derive(Debug, Clone, Default)]
pub(crate) struct SchemaContractRuntimeSubsystem {
    pub(crate) aspect_contract_plans: AspectContractPlanCatalog,
    #[cfg(test)]
    pub(crate) relation_integrity_plans: RelationIntegrityPlanCatalog,
    pub(crate) relation_integrity_registrations: Vec<InvariantRegistration>,
    pub(crate) custom_invariant_registries: FrozenCustomInvariantRegistry,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) initial_custom_invariants_sealed: bool,
}

impl RuntimeSubsystem for SchemaContractRuntimeSubsystem {
    type Config = crate::config::data::RelationalRuntimeConfig;

    fn new(config: &Self::Config) -> Self {
        let relation_integrity_plans = lower_relation_integrity_plans(&config.schema.registry);
        Self {
            aspect_contract_plans: lower_aspect_plans(&config.schema.registry),
            relation_integrity_registrations: relation_integrity_plans
                .relation_plans
                .values()
                .flat_map(crate::validation::data::relation_integrity_registrations_for_plan)
                .collect(),
            custom_invariant_registries: FrozenCustomInvariantRegistry::default(),
            custom_invariant_generation: 0,
            initial_custom_invariants_sealed: false,
            #[cfg(test)]
            relation_integrity_plans,
        }
    }

    fn fork(&self) -> Self {
        self.clone()
    }
}
