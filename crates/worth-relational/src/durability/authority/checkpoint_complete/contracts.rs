//! The existing mirror contract projection is copied under its actual final owner.
use super::Denial;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::RelationalCheckpointManifestDraft;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectShape, PortableAspectContract};
pub(super) fn install(
    schema: &RelationalBranchRootSchemaAuthority,
    draft: &mut RelationalCheckpointManifestDraft,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let contracts = schema.retained_aspect_contracts();
    let mut bytes = std::alloc::Layout::array::<PortableAspectContract>(contracts.len())
        .map_err(|_| overflow())?
        .size();
    for contract in contracts {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        bytes = bytes
            .checked_add(contract.key().as_str().len())
            .ok_or_else(overflow)?;
        if let AspectShape::Struct(shape) = contract.shape() {
            bytes = bytes
                .checked_add(
                    PortableAspectContract::decode_field_roster_storage_bytes(shape.fields().len())
                        .ok_or_else(overflow)?,
                )
                .ok_or_else(overflow)?;
            for field in shape.fields() {
                bytes = bytes
                    .checked_add(field.key().as_str().len())
                    .ok_or_else(overflow)?;
            }
        }
    }
    let memory = if bytes == 0 {
        None
    } else {
        request
            .reserve_retained_memory(u64::try_from(bytes).map_err(|_| overflow())?)
            .map_err(|cause| Denial::Request(cause.into()))?
    };
    let mut copied = Vec::with_capacity(contracts.len());
    for contract in contracts {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        copied.push(PortableAspectContract::from_contract(contract));
    }
    draft.aspect_contracts = copied;
    draft.allocations.aspect_contracts = memory;
    Ok(())
}
fn overflow() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
}
