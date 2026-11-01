//! Signal quotes its own string-bearing recorded initial-root grammar.
use super::*;
use serde_json::value::RawValue;
type Denial = SignalInitialEmptyRootRefusal;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fields<'a> {
    #[serde(rename = "version")]
    _version: serde::de::IgnoredAny,
    #[serde(rename = "root")]
    _root: serde::de::IgnoredAny,
    #[serde(borrow)]
    schema_commitment: &'a RawValue,
    #[serde(borrow)]
    visibility_commitment: &'a RawValue,
    #[serde(borrow)]
    definition_commitment: &'a RawValue,
}
pub(super) fn fixed_bytes() -> Result<usize, Denial> {
    std::mem::size_of::<RecordedInitialEmptyMeaning>()
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
) -> Result<RecordedSignalInitialEmptyRoot, Denial> {
    let work = u64::try_from(bytes.len()).map_err(|_| Denial::Description)?;
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(work))
        .map_err(Denial::Request)?
        .0
        .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    // Borrowed serde keys may still allocate escaped-string scratch. Its byte
    // growth/old allocation/error peak is funded before this first parser.
    let scratch = bytes
        .len()
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(256))
        .ok_or(Denial::Description)?;
    let _scratch = reserve(execution, scratch)?;
    let fields: Fields<'_> = serde_json::from_slice(bytes).map_err(|_| Denial::Description)?;
    let visibility = string_bytes(fields.visibility_commitment)?;
    let definition = string_bytes(fields.definition_commitment)?;
    let heap = fixed_bytes()?
        .checked_add(string_bytes(fields.schema_commitment)?)
        .and_then(|bytes| bytes.checked_add(visibility))
        .and_then(|bytes| bytes.checked_add(definition))
        .ok_or(Denial::Description)?;
    let funding = reserve(execution, heap)?;
    let mut meaning: RecordedInitialEmptyMeaning =
        serde_json::from_slice(bytes).map_err(|_| Denial::Description)?;
    execution.consult().map_err(Denial::Request)?;
    meaning._funding = funding;
    Ok(RecordedSignalInitialEmptyRoot {
        meaning: Arc::new(meaning),
    })
}
