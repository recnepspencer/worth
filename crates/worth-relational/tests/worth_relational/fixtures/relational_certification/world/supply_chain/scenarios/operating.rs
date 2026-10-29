pub(crate) use super::{BaselineName, SupplyChainBaseline};
pub(crate) use crate::fast::relational_certification::world::supply_chain::definition::SupplyChainWorldDefinition;
pub(crate) use crate::fast::relational_certification::world::supply_chain::oracle::{
    OracleBranch, OracleState,
};
pub(crate) use crate::fast::relational_certification::world::supply_chain::scale::SupplyChainScale;

pub(crate) fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let definition = SupplyChainWorldDefinition::operating(scale)
        .expect("canonical operating definition must validate");
    let branch = OracleBranch::genesis(OracleState::from_definition(&definition));
    SupplyChainBaseline {
        name: BaselineName::Operating,
        scale,
        definition,
        branch,
        branch_intents: Vec::new(),
        retention_obligations: Vec::new(),
        pre_upgrade_schema: None,
        post_upgrade_schema: None,
    }
}
