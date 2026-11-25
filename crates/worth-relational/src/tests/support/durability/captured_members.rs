//! In-crate test-only cold source backed strictly by a captured checkpoint's
//! external members (used by `worth-relational` unit tests).
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

use super::{Req, ReqArrayBuilder};
use worth_execution::{ExecutionAllocationPolicy, ExecutionImmutableBytes};

use crate::durability::data::{
    DurableCheckpoint, RelationalCheckpointMember, RelationalNativeCheckpoint,
    RelationalNativeCheckpointCaptureDenial,
};
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
    RelationalDurableMemberRetirementObservation, RelationalDurableMemberRetirementRefusal,
    RelationalDurableOwnedReadRefusal, RelationalDurableReadBounds, RelationalDurableReadPayload,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as StorageCause;

#[derive(Debug, Default)]
pub(crate) struct CapturedMembersColdSource {
    pub(crate) members: BTreeMap<RelationalDurableMemberLocator, ExecutionImmutableBytes>,
    pub(crate) fallback: Option<Arc<dyn RelationalDurableColdSource>>,
}

pub(crate) fn captured_members_source(
    checkpoint: &RelationalNativeCheckpoint,
) -> Arc<dyn RelationalDurableColdSource> {
    captured_members_source_with_fallback(checkpoint, None)
}

pub(crate) fn captured_members_source_with_fallback(
    checkpoint: &RelationalNativeCheckpoint,
    fallback: Option<Arc<dyn RelationalDurableColdSource>>,
) -> Arc<dyn RelationalDurableColdSource> {
    let mut members = BTreeMap::new();
    for member in checkpoint.members() {
        match member {
            RelationalCheckpointMember::New { locator, bytes } => {
                members.insert(*locator, bytes.clone());
            }
            RelationalCheckpointMember::Carried { .. } => {
                assert!(
                    fallback.is_some(),
                    "standalone external capture owns all of its members"
                );
            }
        }
    }
    Arc::new(CapturedMembersColdSource { members, fallback })
}

pub(crate) fn expect_durability_failure(
    stop: crate::durability::data::RelationalRecoveryStop,
) -> crate::durability::data::DurabilityError {
    match stop {
        crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause) => {
            panic!("expected failed durable evidence, received admission image refusal: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Durability(error) => error,
        crate::durability::data::RelationalRecoveryStop::Content(cause) => {
            panic!("unexpected content refusal: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Allocation(cause) => {
            panic!("expected failed durable evidence, received allocation stop: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Request(cause) => {
            panic!("expected failed durable evidence, received request stop: {cause:?}")
        }
    }
}

pub(crate) fn empty_cold_source() -> Arc<dyn RelationalDurableColdSource> {
    Arc::new(CapturedMembersColdSource::default())
}

pub(crate) fn members_from_captured(
    members: &crate::durability::data::CapturedCheckpointMembers,
) -> Arc<dyn RelationalDurableColdSource> {
    let map = members
        .entries()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
            RelationalCheckpointMember::Carried { .. } => {
                panic!("captured checkpoint members must be New for standalone source")
            }
        })
        .collect();
    Arc::new(CapturedMembersColdSource {
        members: map,
        fallback: None,
    })
}

pub(crate) fn encode_checkpoint_with_source(
    execution: &Req<'_, '_>,
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (
        RelationalNativeCheckpoint,
        Arc<dyn RelationalDurableColdSource>,
    ),
    RelationalNativeCheckpointCaptureDenial,
> {
    let (bytes, _, members) =
        crate::durability::log::native_checkpoint_codec::encode_checkpoint_members(
            execution, checkpoint, policy,
        )?;
    let source = members_from_captured(&members);
    let native = RelationalNativeCheckpoint::from_untrusted_bytes(bytes.bytes().to_vec())
        .with_captured_members(members);
    Ok((native, source))
}

impl RelationalDurableColdSource for CapturedMembersColdSource {
    fn retain_members_for_request(
        &self,
        request: &Req<'_, '_>,
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
        _: &Req<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn checkpoint_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<
        (RelationalDurableReadBounds, RelationalDurableReadPayload),
        RelationalDurableOwnedReadRefusal,
    > {
        let original = self.checkpoint_selected_read_protection(request)?;
        Ok((
            RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: PatchStreamPosition(0),
                transitions: crate::branch::RelationalBranchTransitionSequence(0),
            },
            original,
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &Req<'_, '_>,
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
        if let Some(bytes) = self.members.get(&locator) {
            return Ok(bytes.clone());
        }
        if let Some(fallback) = &self.fallback {
            let protection = fallback.checkpoint_selected_read_protection(request)?;
            return fallback.read_member_owned(request, protection.as_ref(), locator, limit);
        }
        Err(RelationalDurableOwnedReadRefusal::Storage(
            StorageCause::MissingMember,
        ))
    }
    fn read_member_prefix_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        let bytes = self.read_member_owned(request, original, locator, locator.length)?;
        let take = length.min(bytes.len());
        let mut out = ReqArrayBuilder::<u8>::allocate(*request, take)
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
        if let Some(bytes) = self.members.get(&locator) {
            return Ok(bytes.to_vec());
        }
        if let Some(fallback) = &self.fallback {
            return fallback.read_member(locator, limit);
        }
        Err(StorageCause::MissingMember)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, StorageCause> {
        Err(StorageCause::SourceUnavailable)
    }
}
