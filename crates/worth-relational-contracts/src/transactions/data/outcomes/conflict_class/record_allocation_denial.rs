use serde::{Deserialize, Serialize};

use crate::history::data::record_allocation_class::RecordAllocationClass;
use crate::identity::data::PartitionId;
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordAllocationDenial {
    GenerationExhausted {
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
    },
    SlotFrontierExhausted {
        class: RecordAllocationClass,
        partition_id: PartitionId,
    },
    ReplayEvidenceMissing {
        ordinal: u64,
    },
    ReplayEvidenceUnexpected {
        expected_ordinal: u64,
        observed_ordinal: u64,
    },
    ReplayTargetMismatch {
        ordinal: u64,
        expected: RecordRef,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    },
    ReplayAppendFrontierMismatch {
        ordinal: u64,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        expected_slot: usize,
        observed_slot: usize,
    },
    ReplaySlotUnavailable {
        ordinal: u64,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
    },
    ReplayGenerationMismatch {
        ordinal: u64,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        expected_generation: u32,
        observed_generation: u32,
    },
    ReplayEvidenceRemaining {
        remaining: usize,
    },
    ArenaWriteDenied {
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        detail: String,
    },
    /// The append slots one commit's live and cancelled evidence names for a
    /// class and partition are not one contiguous block from the frontier: a
    /// hole, a duplicate, or a slot behind the frontier.
    ReplayEvidenceBlockBroken {
        class: RecordAllocationClass,
        partition_id: PartitionId,
        expected_slot: usize,
        observed_slot: usize,
    },
    /// A cancelled slot's recorded generation is not the one its slot holds.
    ReplayCancelledSlotInvalid {
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        expected_generation: u32,
        observed_generation: u32,
    },
    /// A candidate cancelled or ordered a record it holds no live allocation for.
    LiveAllocationUnknown {
        record: RecordRef,
    },
    /// The plan's created records are not exactly the live allocations.
    LiveAllocationOrderDiverged {
        planned: usize,
        live: usize,
    },
}

impl RecordAllocationDenial {
    pub fn detail(&self) -> String {
        match self {
            Self::GenerationExhausted {
                class,
                partition_id,
                slot,
            } => format!(
                "{class:?} generation exhausted for partition {partition_id:?} slot {slot}"
            ),
            Self::SlotFrontierExhausted {
                class,
                partition_id,
            } => format!("{class:?} slot frontier exhausted for partition {partition_id:?}"),
            Self::ReplayEvidenceMissing { ordinal } => {
                format!("canonical replay allocation evidence is missing ordinal {ordinal}")
            }
            Self::ReplayEvidenceUnexpected {
                expected_ordinal,
                observed_ordinal,
            } => format!(
                "canonical replay allocation ordinal {observed_ordinal} does not match expected ordinal {expected_ordinal}"
            ),
            Self::ReplayTargetMismatch {
                ordinal,
                expected,
                class,
                partition_id,
            } => format!(
                "canonical replay allocation {ordinal} targets {expected:?}, not {class:?} partition {partition_id:?}"
            ),
            Self::ReplayAppendFrontierMismatch {
                ordinal,
                class,
                partition_id,
                expected_slot,
                observed_slot,
            } => format!(
                "canonical replay allocation {ordinal} selects {class:?} partition {partition_id:?} append slot {observed_slot}, expected frontier {expected_slot}"
            ),
            Self::ReplaySlotUnavailable {
                ordinal,
                class,
                partition_id,
                slot,
            } => format!(
                "canonical replay allocation {ordinal} selects unavailable {class:?} partition {partition_id:?} slot {slot}"
            ),
            Self::ReplayGenerationMismatch {
                ordinal,
                class,
                partition_id,
                slot,
                expected_generation,
                observed_generation,
            } => format!(
                "canonical replay allocation {ordinal} selects {class:?} partition {partition_id:?} slot {slot} generation {observed_generation}, expected {expected_generation}"
            ),
            Self::ReplayEvidenceRemaining { remaining } => format!(
                "canonical replay retained {remaining} unconsumed record allocation decisions"
            ),
            Self::ArenaWriteDenied {
                class,
                partition_id,
                slot,
                detail,
            } => format!(
                "{class:?} allocation for partition {partition_id:?} slot {slot} was denied: {detail}"
            ),
            Self::ReplayEvidenceBlockBroken {
                class,
                partition_id,
                expected_slot,
                observed_slot,
            } => format!(
                "canonical replay {class:?} partition {partition_id:?} append evidence is not one contiguous block: slot {observed_slot}, expected {expected_slot}"
            ),
            Self::ReplayCancelledSlotInvalid {
                class,
                partition_id,
                slot,
                expected_generation,
                observed_generation,
            } => format!(
                "canonical replay cancels {class:?} partition {partition_id:?} slot {slot} at generation {expected_generation}, which holds {observed_generation}"
            ),
            Self::LiveAllocationUnknown { record } => {
                format!("coupled allocation evidence holds no live allocation of {record:?}")
            }
            Self::LiveAllocationOrderDiverged { planned, live } => format!(
                "coupled plan creates {planned} records but {live} live allocations remain"
            ),
        }
    }
}
