#[cfg(test)]
use std::sync::Arc;

use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

/// Inline artifacts implement the same read port as Store. This keeps the
/// member verification and loading path independent of artifact placement.
#[derive(Debug)]
pub(super) struct InlineCheckpointSource {
    pub(super) members: worth_execution::ExecutionOrderedMap<
        RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    >,
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
                selected_checkpoint: true,
                canonical: PatchStreamPosition(0),
                transitions: crate::branch::RelationalBranchTransitionSequence(0),
            },
            Box::new(()),
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        self.checkpoint_read_protection(request)
            .map(|(_, original)| original)
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
        Ok(output.seal_immutable_bytes().map_err(Refusal::Buffer)?)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), crate::durability::medium::RelationalDurableMemberRetirementRefusal> {
        // These actual immutable payloads own bytes/metadata only, with no physical pin or registration.
        if original.is::<()>() {
            Ok(())
        } else {
            Err(crate::durability::medium::RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> crate::durability::medium::RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => {
                crate::durability::medium::RelationalDurableMemberRetirementObservation::Complete
            }
            Err(cause) => {
                crate::durability::medium::RelationalDurableMemberRetirementObservation::Refused(
                    cause,
                )
            }
        }
    }
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        crate::durability::medium::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
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

    fn read_commit(&self, _position: PatchStreamPosition, _limit: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
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
