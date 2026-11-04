//! Version preflight reads a bounded prefix without claiming payload verification.
use super::{Cause, StoreColdSource};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::durability::{
    RelationalDurableMemberLocator, RelationalDurableOwnedReadRefusal as Refusal,
};

impl StoreColdSource {
    pub(super) fn owned_member_prefix(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        if length as u64 > locator.length {
            return Err(Refusal::Bounds);
        }
        let mut stream = self
            .open_selected_owned_member(request, original, locator, locator.length)?
            .ok_or(Refusal::ForeignProtection)?;
        let mut bytes =
            ExecutionRequestArrayBuilder::allocate(*request, length).map_err(Refusal::Buffer)?;
        let mut scratch = [0u8; 256];
        while bytes.len() < length {
            request.consult().map_err(Refusal::from)?;
            let remaining = (length - bytes.len()).min(scratch.len());
            let count = stream
                .read_next(&mut scratch[..remaining])
                .map_err(|cause| {
                    Refusal::Storage(super::member_read::member_failure(cause, locator.length))
                })?;
            if count == 0 {
                return Err(Refusal::Storage(Cause::Damaged));
            }
            bytes
                .extend_from_slice(&scratch[..count])
                .map_err(Refusal::Buffer)?;
        }
        bytes.seal_immutable_bytes().map_err(Refusal::Buffer)
    }
}
