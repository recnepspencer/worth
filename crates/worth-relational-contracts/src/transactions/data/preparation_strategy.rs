use super::preparation_strategy_posture::{
    ParallelLegality, ParallelProfitability, PreparationStrategySelection, SerialPreparationReason,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparationStrategy {
    pub parallel_legality: ParallelLegality,
    pub parallel_profitability: ParallelProfitability,
    pub selected_mode: PreparationStrategySelection,
    pub serial_selection_reason: Option<SerialPreparationReason>,
}

impl PreparationStrategy {
    pub fn serial(reason: SerialPreparationReason) -> Self {
        Self {
            parallel_legality: ParallelLegality::RequiresSerial,
            parallel_profitability: ParallelProfitability::NotProfitable,
            selected_mode: PreparationStrategySelection::Serial,
            serial_selection_reason: Some(reason),
        }
    }
}
