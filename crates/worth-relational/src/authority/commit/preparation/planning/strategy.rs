use serde::{Deserialize, Serialize};
use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionPosture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelLegality {
    ProvenParallel,
    RequiresSerial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelProfitability {
    Profitable,
    NotProfitable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreparationStrategySelection {
    Serial,
    StagedParallel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerialPreparationReason {
    NoLease,
    SerialPosture,
    ProofRequiresSerial,
    InsufficientPacketBreadth,
    ProfitabilityThreshold,
}

impl SerialPreparationReason {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::NoLease => "no_lease",
            Self::SerialPosture => "serial_posture",
            Self::ProofRequiresSerial => "proof_requires_serial",
            Self::InsufficientPacketBreadth => "insufficient_packet_breadth",
            Self::ProfitabilityThreshold => "profitability_threshold",
        }
    }
}

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
