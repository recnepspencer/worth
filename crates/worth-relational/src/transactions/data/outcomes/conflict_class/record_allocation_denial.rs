use serde::{Deserialize, Serialize};

use crate::history::data::RecordAllocationClass;
use crate::identity::data::PartitionId;
use crate::transactions::data::RecordRef;

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
}

impl RecordAllocationDenial {
    pub(crate) fn detail(&self) -> String {
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
        }
    }
}
