use super::super::types::BlobStreamingResumeAdmission;
use super::super::verification::request_match;
use super::bind_resume_session;
use crate::{
    BlobStreamingChunkWriter, BlobStreamingIngest, BlobStreamingIngestDenial,
    BlobStreamingIngestRequest, BlobStreamingPressureAdmission, BlobStreamingSourceFrame,
    BlobStreamingWindow,
};
use worth_store_budgets::CounterEvidenceStrength;

pub fn run_resumable_streaming_ingest<W>(
    request: BlobStreamingIngestRequest,
    resume_admission: BlobStreamingResumeAdmission,
    window: BlobStreamingWindow,
    pressure: BlobStreamingPressureAdmission,
    counter_strength: CounterEvidenceStrength,
    source_frames: impl IntoIterator<Item = BlobStreamingSourceFrame>,
    writer: &mut W,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial>
where
    W: BlobStreamingChunkWriter,
{
    request_match::verify_resume_request_matches(&resume_admission, &request)?;
    let ingest = BlobStreamingIngest::verify_bounded_content(
        request,
        window,
        pressure,
        counter_strength,
        source_frames,
        writer,
    )?;
    Ok(bind_resume_session::bind_resume_session(
        ingest,
        resume_admission,
    ))
}
