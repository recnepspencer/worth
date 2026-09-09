use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use serde::{Deserialize, Serialize};

use crate::data::node::NodeEvaluationConfig;

/// Installed node meaning retained with the selected execution basis.
/// A partition activates its retained definitions and restores the prior lane on exit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct NodeDefinitionData {
    #[serde(default)]
    pub(crate) tombstoned: bool,
    #[serde(default)]
    pub(crate) conditional_contract_generation: u64,
    #[serde(default)]
    pub(crate) conditional_contract_occurrence: u64,
    #[serde(default)]
    pub(crate) eval_config: NodeEvaluationConfig,
}

impl RetainedStorageMeasurement for NodeDefinitionData {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            tombstoned: _,
            conditional_contract_generation: _,
            conditional_contract_occurrence: _,
            eval_config,
        } = self;
        eval_config.retained_heap_charge(work)
    }
}

#[cfg(test)]
mod retained_charge_tests;
