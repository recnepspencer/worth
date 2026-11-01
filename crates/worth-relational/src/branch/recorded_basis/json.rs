//! Owner-shaped heap admission precedes typed JSON allocation.
use super::*;
mod quote;
pub(super) fn decode(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<RecordedRelationalComponentReference, RelationalRecordedComponentRefusal> {
    use RelationalRecordedComponentRefusal as Refusal;
    let work = u64::try_from(bytes.len()).map_err(|_| copying::malformed())?;
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(work))
        .map_err(|cause| Refusal::Request(cause.into()))?
        .0
        .map_err(|cause| {
            Refusal::Request(worth_execution::WorkCeilingDenial::Stopped(cause).into())
        })?;
    // serde_json scratch grows as a byte Vec and may coexist with its old
    // allocation and an error message. Admit that temporary peak BEFORE keys,
    // escaped strings or errors enter even the borrowed prequote parser.
    let scratch_bytes = bytes
        .len()
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(256))
        .ok_or_else(copying::malformed)?;
    let _scratch = copying::reserve(execution, scratch_bytes)?;
    let quote = quote::quote(bytes)?;
    let _growth = copying::reserve(execution, quote.reallocation)?;
    let funding = copying::reserve(execution, quote.retained)?;
    let mut meaning: RecordedComponentMeaning =
        serde_json::from_slice(bytes).map_err(|_| copying::malformed())?;
    crate::execution::check_request(execution).map_err(Refusal::Request)?;
    meaning._funding = funding;
    Ok(RecordedRelationalComponentReference {
        meaning: Arc::new(meaning),
    })
}
