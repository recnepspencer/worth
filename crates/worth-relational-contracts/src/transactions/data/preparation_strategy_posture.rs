use serde::{Deserialize, Serialize};

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
