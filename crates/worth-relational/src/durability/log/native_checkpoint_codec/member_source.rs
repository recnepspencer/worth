use std::collections::BTreeMap;
use std::sync::Arc;

use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

/// Inline artifacts implement the same read port as Store. This keeps the
/// member verification and loading path independent of artifact placement.
#[derive(Debug)]
pub(super) struct InlineCheckpointSource {
    pub(super) members:
        BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    pub(super) envelopes: BTreeMap<PatchStreamPosition, Arc<[u8]>>,
}

impl RelationalDurableColdSource for InlineCheckpointSource {
    // This immutable image has owned members, not an original journal prefix.
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            crate::durability::medium::RelationalDurableReadBounds,
            crate::durability::medium::RelationalDurableReadPayload,
        ),
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        request.check_live().map_err(|cause| {
            crate::durability::medium::RelationalDurableOwnedReadRefusal::Request(cause.into())
        })?;
        Ok((
            crate::durability::medium::RelationalDurableReadBounds {
                canonical: PatchStreamPosition(0),
                transitions: crate::branch::RelationalBranchTransitionSequence(0),
            },
            Box::new(()),
        ))
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        use sha2::{Digest, Sha256};
        if !original.is::<()>() {
            return Err(Refusal::ForeignProtection);
        }
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        if locator.length > limit {
            return Err(Refusal::Storage(Cause::LimitExceeded {
                length: locator.length,
                limit,
            }));
        }
        let bytes = self
            .members
            .get(&locator)
            .ok_or(Refusal::Storage(Cause::MissingMember))?;
        if bytes.len() as u64 != locator.length
            || <[u8; 32]>::from(Sha256::digest(bytes.bytes())) != locator.sha256
        {
            return Err(Refusal::Storage(Cause::IntegrityMismatch));
        }
        let mut output =
            worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(*request, bytes.len())
                .map_err(Refusal::Buffer)?;
        output
            .extend_from_slice(bytes.bytes())
            .map_err(Refusal::Buffer)?;
        Ok(output
            .seal()
            .map_err(Refusal::Buffer)?
            .into_immutable_bytes())
    }

    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        bounded(self.members.get(&locator).map(AsRef::as_ref), limit)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        bounded(self.envelopes.get(&position).map(AsRef::as_ref), limit)
    }
}

fn bounded(bytes: Option<&[u8]>, limit: u64) -> Result<Vec<u8>, Cause> {
    let bytes = bytes.ok_or(Cause::SourceUnavailable)?;
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(bytes.len())
        .map_err(|_| Cause::MemoryExhausted)?;
    result.extend_from_slice(bytes);
    Ok(result)
}

#[cfg(test)]
mod tests;
