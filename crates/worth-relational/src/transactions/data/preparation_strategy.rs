use super::preparation_strategy_posture::{
    ParallelLegality, ParallelProfitability, PreparationStrategySelection, SerialPreparationReason,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparationStrategy {
    pub(crate) parallel_legality: ParallelLegality,
    pub(crate) parallel_profitability: ParallelProfitability,
    pub(crate) selected_mode: PreparationStrategySelection,
    pub(crate) serial_selection_reason: Option<SerialPreparationReason>,
}

impl PreparationStrategy {
    pub(crate) fn serial(reason: SerialPreparationReason) -> Self {
        Self {
            parallel_legality: ParallelLegality::RequiresSerial,
            parallel_profitability: ParallelProfitability::NotProfitable,
            selected_mode: PreparationStrategySelection::Serial,
            serial_selection_reason: Some(reason),
        }
    }
}
