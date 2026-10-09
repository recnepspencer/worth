use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::{
    foreground_reservation::ForegroundIoLaneKind, BackgroundIoPressureClass, BackgroundPacingDenial,
};

use crate::{BlobChunkIntegrityDenial, BlobStreamingIngestCounterSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobStreamingIngestDenial {
    EmptyStreamingWindow,
    EmptySourceFrame,
    EmptyDeclaredObject,
    WindowExceedsResidentEnvelope {
        window_bytes: u64,
        envelope_bytes: u64,
    },
    WholeObjectMaterializationRejected {
        bytes: u64,
    },
    SourceWindowExceedsResidentEnvelope {
        window_bytes: u64,
        envelope_bytes: u64,
    },
    MissingExactCounters {
        actual: CounterEvidenceStrength,
    },
    AllocationWindowExceeded {
        window_bytes: u64,
        allocation_bytes: u64,
    },
    BackgroundPressureYielded {
        counters: BlobStreamingIngestCounterSnapshot,
    },
    BackgroundPressureDeferred {
        counters: BlobStreamingIngestCounterSnapshot,
    },
    BackgroundPressureDenied {
        source: BackgroundPacingDenial,
        counters: BlobStreamingIngestCounterSnapshot,
    },
    BackgroundPressureThrottledWithoutAdmittedCapacity {
        counters: BlobStreamingIngestCounterSnapshot,
    },
    BackgroundPressureViolation {
        counters: BlobStreamingIngestCounterSnapshot,
    },
    BackgroundPressureClassMismatch {
        actual: BackgroundIoPressureClass,
    },
    ForegroundReservationNotAdmitted {
        lane: ForegroundIoLaneKind,
    },
    ForegroundReservationLaneMismatch {
        lane: ForegroundIoLaneKind,
    },
    ScalarBackendCertificationRejected,
    BackendWriteOrdinalMismatch {
        expected: u64,
        actual: u64,
    },
    BackendWriteBytesMismatch {
        expected: u64,
        actual: u64,
    },
    BackendWritePayloadMismatch {
        ordinal: u64,
    },
    ResumeSessionRequestMismatch,
    ChunkIntegrity(BlobChunkIntegrityDenial),
}

impl From<BlobChunkIntegrityDenial> for BlobStreamingIngestDenial {
    fn from(denial: BlobChunkIntegrityDenial) -> Self {
        Self::ChunkIntegrity(denial)
    }
}

pub fn reject_full_blob_vec_as_streaming_ingest(bytes: Vec<u8>) -> BlobStreamingIngestDenial {
    BlobStreamingIngestDenial::WholeObjectMaterializationRejected {
        bytes: bytes.len() as u64,
    }
}
