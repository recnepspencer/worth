//! Bridge owns the heap quote for its exact three baseline strings.
use super::*;
use serde_json::value::RawValue;
type Denial = RuntimeWorldBaselineRefusal;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fields<'a> {
    #[serde(rename = "version")]
    _version: serde::de::IgnoredAny,
    #[serde(borrow)]
    baseline: &'a RawValue,
    #[serde(borrow)]
    adapter_semantic_identity: &'a RawValue,
    #[serde(borrow)]
    graph_role: &'a RawValue,
}
pub(super) fn fixed_bytes() -> Result<usize, Denial> {
    std::mem::size_of::<RecordedBaselineMeaning>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .ok_or(Denial::Description)
}
pub(super) fn reserve(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
    execution
        .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Denial::Description)?)
        .map_err(Denial::Request)
}
fn string_bytes(value: &RawValue) -> Result<usize, Denial> {
    let bytes = value.get().as_bytes();
    if bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(Denial::Description);
    }
    bytes.len().checked_sub(2).ok_or(Denial::Description)
}
pub(super) fn decode(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<RecordedRuntimeWorldBaseline, Denial> {
    let work = u64::try_from(bytes.len()).map_err(|_| Denial::Description)?;
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(work))
        .map_err(Denial::Request)?
        .0
        .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    let scratch = bytes
        .len()
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(256))
        .ok_or(Denial::Description)?;
    let _scratch = reserve(execution, scratch)?;
    let fields: Fields<'_> = serde_json::from_slice(bytes).map_err(|_| Denial::Description)?;
    let adapter = string_bytes(fields.adapter_semantic_identity)?;
    let role = string_bytes(fields.graph_role)?;
    let heap = fixed_bytes()?
        .checked_add(string_bytes(fields.baseline)?)
        .and_then(|bytes| bytes.checked_add(adapter))
        .and_then(|bytes| bytes.checked_add(role))
        .ok_or(Denial::Description)?;
    let funding = reserve(execution, heap)?;
    let mut meaning: RecordedBaselineMeaning =
        serde_json::from_slice(bytes).map_err(|_| Denial::Description)?;
    execution.consult().map_err(Denial::Request)?;
    meaning._funding = funding;
    Ok(RecordedRuntimeWorldBaseline {
        meaning: Arc::new(meaning),
    })
}
