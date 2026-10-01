use worth_store_budgets::CounterEvidenceStrength;

use crate::{
    BlobChunkByteRange, BlobChunkOrdinal, BlobCorruptionDenial, BlobDamageCase,
    BlobQuarantineDiagnostics, BlobStreamingReadCounterSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobStreamingReadDenial {
    EmptyReadWindow,
    EmptyObservedReadChunk,
    ReadWindowExceedsResidentEnvelope {
        window_bytes: u64,
        envelope_bytes: u64,
    },
    WholeObjectExpectedBufferRejected {
        bytes: u64,
    },
    MissingExactCounters {
        actual: CounterEvidenceStrength,
    },
    MissingChunk {
        ordinal: BlobChunkOrdinal,
        counters: BlobStreamingReadCounterSnapshot,
    },
    ReorderedChunk {
        expected: BlobChunkOrdinal,
        actual: BlobChunkOrdinal,
        counters: BlobStreamingReadCounterSnapshot,
    },
    ChunkRangeMismatch {
        ordinal: BlobChunkOrdinal,
        expected: BlobChunkByteRange,
        actual: BlobChunkByteRange,
        counters: BlobStreamingReadCounterSnapshot,
    },
    CorruptedChunk {
        ordinal: BlobChunkOrdinal,
        damage_case: BlobDamageCase,
        diagnostics: Box<BlobQuarantineDiagnostics>,
        counters: BlobStreamingReadCounterSnapshot,
    },
    ColdChunkUnavailable {
        ordinal: BlobChunkOrdinal,
        counters: BlobStreamingReadCounterSnapshot,
    },
    ExtraChunk {
        ordinal: BlobChunkOrdinal,
        counters: BlobStreamingReadCounterSnapshot,
    },
    CorruptionReferenceEdgeMismatch(Box<BlobCorruptionDenial>),
    LogicalContentDigestMismatch,
    ChunkTreeRootMismatch,
}

pub fn reject_full_blob_vec_as_streaming_read(bytes: Vec<u8>) -> BlobStreamingReadDenial {
    BlobStreamingReadDenial::WholeObjectExpectedBufferRejected {
        bytes: bytes.len() as u64,
    }
}
