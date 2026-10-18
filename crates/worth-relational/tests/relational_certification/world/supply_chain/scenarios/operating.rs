use super::{BaselineName, SupplyChainBaseline};
use crate::relational_certification::world::supply_chain::definition::SupplyChainWorldDefinition;
use crate::relational_certification::world::supply_chain::oracle::{OracleBranch, OracleState};
use crate::relational_certification::world::supply_chain::scale::SupplyChainScale;

pub(super) fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
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
