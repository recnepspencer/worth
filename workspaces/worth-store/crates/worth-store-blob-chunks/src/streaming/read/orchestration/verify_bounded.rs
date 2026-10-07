use worth_store_budgets::CounterEvidenceStrength;

use super::super::transitions::{finish_verified_read, observe_chunk_window};
use super::super::types::BlobStreamingVerifiedRead;
use super::super::verification::{counter_strength, StreamingReadVerifier};
use crate::{
    BlobQuarantineAuthority, BlobStreamingReadCounterSnapshot, BlobStreamingReadDenial,
    BlobStreamingReadObservation, BlobStreamingReadRequest, BlobStreamingReadWindow,
};

impl BlobStreamingVerifiedRead {
    /// Verifies chunk ordering and content. This does not admit physical reads or
    /// establish Store residency; the Store runtime owns those decisions.
    pub fn verify_bounded_content(
        request: BlobStreamingReadRequest,
        window: BlobStreamingReadWindow,
        quarantine_authority: BlobQuarantineAuthority,
        counter_strength: CounterEvidenceStrength,
        observations: impl IntoIterator<Item = BlobStreamingReadObservation>,
    ) -> Result<Self, BlobStreamingReadDenial> {
        counter_strength::require_exact(counter_strength)?;
        let mut counters = BlobStreamingReadCounterSnapshot::start(counter_strength);
        let mut verifier = StreamingReadVerifier::new(request, window, quarantine_authority);
        for observation in observations {
            observe_chunk_window::observe_chunk_window(&mut verifier, observation, &mut counters)?;
        }
        finish_verified_read::finish_verified_read(verifier, counters)
    }
}
