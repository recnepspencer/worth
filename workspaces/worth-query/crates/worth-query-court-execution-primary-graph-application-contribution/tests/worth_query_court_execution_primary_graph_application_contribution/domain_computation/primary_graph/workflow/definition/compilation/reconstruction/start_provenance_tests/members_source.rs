//! Member-backed cold source for standalone external checkpoint capture/restore.
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::{
    RelationalCheckpointMember, RelationalDurableColdSource, RelationalDurableMemberCustody,
    RelationalDurableMemberLocator, RelationalDurableMemberRetirementObservation,
    RelationalDurableMemberRetirementRefusal, RelationalDurableOwnedReadRefusal,
    RelationalDurableReadBounds, RelationalDurableReadPayload, RelationalNativeCheckpoint,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as StorageCause;

#[derive(Debug)]
pub(super) struct CapturedMembersSource {
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, ExecutionImmutableBytes>,
}

pub(super) fn from_checkpoint(
    checkpoint: &RelationalNativeCheckpoint,
) -> Arc<dyn RelationalDurableColdSource> {
    let members = checkpoint
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
            RelationalCheckpointMember::Carried { .. } => {
                panic!("standalone external capture owns all of its members")
            }
        })
        .collect();
    Arc::new(CapturedMembersSource { members })
}

impl RelationalDurableColdSource for CapturedMembersSource {
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if original.is::<()>() {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _: &ExecutionRequest<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn checkpoint_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (RelationalDurableReadBounds, RelationalDurableReadPayload),
        RelationalDurableOwnedReadRefusal,
    > {
        let original = self.checkpoint_selected_read_protection(request)?;
        Ok((
            RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: PatchStreamPosition(0),
                transitions: RelationalBranchTransitionSequence(0),
            },
            original,
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        if !original.is::<()>() {
            return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
        }
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        if locator.length > limit {
            return Err(RelationalDurableOwnedReadRefusal::Storage(
                StorageCause::LimitExceeded {
                    length: locator.length,
                    limit,
                },
            ));
        }
        self.members
            .get(&locator)
            .cloned()
            .ok_or(RelationalDurableOwnedReadRefusal::Storage(
                StorageCause::MissingMember,
            ))
    }
    fn read_member_prefix_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        let bytes = self.read_member_owned(request, original, locator, locator.length)?;
        let take = length.min(bytes.len());
        let mut out = ExecutionRequestArrayBuilder::<u8>::allocate(*request, take)
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        out.extend_from_slice(&bytes[..take])
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        out.seal_immutable_bytes()
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)
    }
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, StorageCause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, StorageCause> {
        if locator.length > limit {
            return Err(StorageCause::LimitExceeded {
                length: locator.length,
                limit,
            });
        }
        self.members
            .get(&locator)
            .map(|bytes| bytes.to_vec())
            .ok_or(StorageCause::MissingMember)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, StorageCause> {
        Err(StorageCause::SourceUnavailable)
    }
}
