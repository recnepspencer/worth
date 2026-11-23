//! Record the selected index build strategy on the installed runtime.
use crate::authority::commit::preparation::planning::strategy::{
    ParallelLegality, ParallelProfitability, PreparationStrategy, PreparationStrategySelection,
};
use crate::runtime::RelationalRuntime;

pub(super) fn record_index_preparation_strategy_counters(
    runtime: &RelationalRuntime,
    definition_count: usize,
    strategy: &PreparationStrategy,
) {
    runtime.performance_access().count_preparation_packet_shape(
        definition_count,
        definition_count,
        usize::from(definition_count != 0),
        usize::from(definition_count != 0),
    );
    if strategy.parallel_legality == ParallelLegality::ProvenParallel {
        runtime
            .performance_access()
            .count_preparation_parallel_legal();
    }
    if strategy.parallel_profitability == ParallelProfitability::Profitable {
        runtime
            .performance_access()
            .count_preparation_parallel_profitable();
    }
    match strategy.selected_mode {
        PreparationStrategySelection::Serial => runtime
            .performance_access()
            .count_preparation_serial_strategy(),
        PreparationStrategySelection::StagedParallel => runtime
            .performance_access()
            .count_preparation_staged_parallel_strategy(),
    }
}
