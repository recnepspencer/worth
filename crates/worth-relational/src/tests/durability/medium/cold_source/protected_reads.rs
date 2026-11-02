//! Selected custody over this fixture's immutable member map; no eager payload reads.
pub(super) use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

pub(super) fn validate_retirement(
    original: &(dyn std::any::Any + Send + Sync),
) -> Result<(), crate::durability::medium::RelationalDurableMemberRetirementRefusal> {
    if original.is::<()>() {
        Ok(())
    } else {
        Err(crate::durability::medium::RelationalDurableMemberRetirementRefusal::ForeignCustody)
    }
}

pub(super) fn validate(
    request: &ExecutionRequest<'_, '_>,
    original: &(dyn std::any::Any + Send + Sync),
) -> Result<(), Refusal> {
    request
        .check_live()
        .map_err(|cause| Refusal::Request(cause.into()))?;
    if !original.is::<()>() {
        return Err(Refusal::ForeignProtection);
    }
    Ok(())
}

pub(super) fn damaged_copy(
    request: &ExecutionRequest<'_, '_>,
    bytes: &ExecutionImmutableBytes,
) -> Result<ExecutionImmutableBytes, Refusal> {
    let mut copy =
        ExecutionRequestArrayBuilder::allocate(*request, bytes.len()).map_err(Refusal::Buffer)?;
    copy.push(bytes[0] ^ 1).map_err(Refusal::Buffer)?;
    copy.extend_from_slice(&bytes[1..])
        .map_err(Refusal::Buffer)?;
    copy.seal_immutable_bytes().map_err(Refusal::Buffer)
}

pub(super) fn copy(
    request: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<ExecutionImmutableBytes, Refusal> {
    let mut copy =
        ExecutionRequestArrayBuilder::allocate(*request, bytes.len()).map_err(Refusal::Buffer)?;
    copy.extend_from_slice(bytes).map_err(Refusal::Buffer)?;
    copy.seal_immutable_bytes().map_err(Refusal::Buffer)
}
