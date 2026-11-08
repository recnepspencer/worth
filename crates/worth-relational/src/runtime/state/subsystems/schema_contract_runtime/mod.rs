use crate::runtime::state::subsystems::RuntimeSubsystem;
use crate::schema::data::AspectContractPlanCatalog;
#[cfg(test)]
use crate::schema::data::RelationIntegrityPlanCatalog;
use crate::schema::{lower_aspect_plans, lower_relation_integrity_plans};
use crate::validation::data::InvariantRegistration;
use crate::validation::FrozenCustomInvariantRegistry;

#[derive(Debug, Clone)]
pub(crate) struct SchemaContractRuntimeSubsystem {
    pub(crate) invariant_points: std::sync::Arc<
        [Vec<InvariantRegistration>; crate::validation::data::InvariantExecutionPoint::COUNT],
    >,
    pub(crate) schema_authority_digest: [u8; 32],
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
        let relation_integrity_registrations: Vec<_> = relation_integrity_plans
            .relation_plans
            .values()
            .flat_map(crate::validation::data::relation_integrity_registrations_for_plan)
            .collect();
        let invariant_points = std::sync::Arc::new(std::array::from_fn(|point| {
            config
                .schema
                .invariant_catalog
                .registrations
                .iter()
                .chain(&relation_integrity_registrations)
                .filter(|r| {
                    point == r.execution_point as usize
                        || (point
                            == crate::validation::data::InvariantExecutionPoint::ProposedState
                                as usize
                            && crate::validation::data::InvariantExecutionPoint::ProposedState
                                .includes(r.execution_point))
                })
                .cloned()
                .collect()
        }));
        Self {
            schema_authority_digest: config.schema.registry.authority_digest_bytes(),
            aspect_contract_plans: lower_aspect_plans(&config.schema.registry),
            invariant_points,
            relation_integrity_registrations,
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
