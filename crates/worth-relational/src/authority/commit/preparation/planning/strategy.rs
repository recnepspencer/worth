use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionPosture;

pub use crate::transactions::data::preparation_strategy_posture::{
    ParallelLegality, ParallelProfitability, PreparationStrategySelection, SerialPreparationReason,
};

pub use crate::transactions::data::preparation_strategy::PreparationStrategy;

pub(crate) const MIN_PARALLEL_PACKET_WIDTH: usize = 2;
pub(crate) const TARGET_PREPARATION_ITEMS_PER_PACKET: usize = 32;

pub(crate) const fn packet_width_is_profitable(
    packet_count: usize,
    min_parallel_packet_width: usize,
) -> bool {
    packet_count >= min_parallel_packet_width
}

pub(crate) const fn coarse_preparation_packet_count(
    item_count: usize,
    target_items_per_packet: usize,
) -> usize {
    if item_count == 0 {
        0
    } else {
        item_count.div_ceil(target_items_per_packet)
    }
}

pub(crate) fn strategy_for_parallel_packets(
    lease: Option<&ExecutionResourceLease>,
    packet_count: usize,
) -> PreparationStrategy {
    if lease.is_none() {
        return PreparationStrategy::serial(SerialPreparationReason::NoLease);
    }
    if lease.is_some_and(|lease| lease.resolved_posture() == ExecutionPosture::Serial) {
        return PreparationStrategy::serial(SerialPreparationReason::SerialPosture);
    }

    if !packet_width_is_profitable(packet_count, MIN_PARALLEL_PACKET_WIDTH) {
        return PreparationStrategy {
            parallel_legality: ParallelLegality::ProvenParallel,
            parallel_profitability: ParallelProfitability::NotProfitable,
            selected_mode: PreparationStrategySelection::Serial,
            serial_selection_reason: Some(SerialPreparationReason::InsufficientPacketBreadth),
        };
    }

    PreparationStrategy {
        parallel_legality: ParallelLegality::ProvenParallel,
        parallel_profitability: ParallelProfitability::Profitable,
        selected_mode: PreparationStrategySelection::StagedParallel,
        serial_selection_reason: None,
    }
}
