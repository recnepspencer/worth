#[cfg(test)]
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};

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
            worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadBounds,
            worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadPayload,
        ),
        worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    >{
        request.check_live().map_err(|cause| {
            worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal::Request(cause.into())
        })?;
        Ok((
            worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: PatchStreamPosition(0),
                transitions:
                    contracts::branch::transition::sequence::RelationalBranchTransitionSequence(0),
            },
            Box::new(()),
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadPayload,
        worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    >{
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
        worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    >{
        use sha2::{Digest, Sha256};
        use worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal as Refusal;
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
        output.seal_immutable_bytes().map_err(Refusal::Buffer)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementRefusal>{
        // These actual immutable payloads own bytes/metadata only, with no physical pin or registration.
        if original.is::<()>() {
            Ok(())
        } else {
            Err(worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation{
        match self.validate_member_custody_retirement(original) {
            Ok(()) => {
                worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation::Complete
            }
            Err(cause) => {
                worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation::Refused(
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
        worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody,
        worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    >{
        worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
    }
    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody, Cause>{
        Ok(worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody::new(()))
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
pub mod tests;
