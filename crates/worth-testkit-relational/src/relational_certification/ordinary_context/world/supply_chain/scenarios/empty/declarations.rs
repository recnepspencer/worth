pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};

pub fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let definition = crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::empty(scale);
    let branch = crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch::genesis(crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState::empty(definition.schema.clone()));
    SupplyChainBaseline {
        name: crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::EmptyInstallation,
        scale,
        definition,
        branch,
        branch_intents: Vec::new(),
        retention_obligations: Vec::new(),
        pre_upgrade_schema: None,
        post_upgrade_schema: None,
    }
}