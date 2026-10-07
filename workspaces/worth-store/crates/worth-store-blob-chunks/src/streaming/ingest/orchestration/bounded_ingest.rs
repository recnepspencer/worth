use worth_store_budgets::CounterEvidenceStrength;

use super::super::transitions::{
    admit_stream, advance_frontier, emit_ingest_receipt, finalize_sequence,
};
use super::super::types::BlobStreamingIngest;
use super::super::verification::counter_strength;
use crate::{
    BlobStreamingChunkWriter, BlobStreamingIngestDenial, BlobStreamingIngestRequest,
    BlobStreamingPressureAdmission, BlobStreamingSourceFrame, BlobStreamingWindow,
};

impl BlobStreamingIngest {
    /// Verifies a bounded content sequence. Physical allocation and durable publication
    /// belong to the Store runtime, not to this mechanism result.
    pub fn verify_bounded_content<W>(
        request: BlobStreamingIngestRequest,
        window: BlobStreamingWindow,
        pressure: BlobStreamingPressureAdmission,
        counter_strength: CounterEvidenceStrength,
        source_frames: impl IntoIterator<Item = BlobStreamingSourceFrame>,
        writer: &mut W,
    ) -> Result<Self, BlobStreamingIngestDenial>
    where
        W: BlobStreamingChunkWriter,
    {
        verify_bounded_content(
            request,
            window,
            pressure,
            counter_strength,
            source_frames,
            writer,
        )
    }
}

pub(crate) fn verify_bounded_content<W>(
    request: BlobStreamingIngestRequest,
    window: BlobStreamingWindow,
    pressure: BlobStreamingPressureAdmission,
    counter_strength: CounterEvidenceStrength,
    source_frames: impl IntoIterator<Item = BlobStreamingSourceFrame>,
    writer: &mut W,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial>
where
    W: BlobStreamingChunkWriter,
{
    counter_strength::require_exact(counter_strength)?;
    let declared_total_bytes = request.declared_total_bytes();
    let (admission, chunking, counters) = admit_stream::admit_stream(request, pressure)?;
    let (admission, chunking, counters) = advance_frontier::advance_frontier(
        source_frames,
        window,
        declared_total_bytes,
        admission,
        chunking,
        counters,
        writer,
    )?;
    let (sequence, counters) =
        finalize_sequence::finalize_sequence(chunking, admission, counters, writer)?;
    emit_ingest_receipt::emit_ingest_receipt(sequence, counters)
}
